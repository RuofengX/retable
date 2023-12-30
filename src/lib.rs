//! A in-memory key-value store.
//! 
//! Designed to be used in multi-threaded environment with high concurrency,
//! like game.

mod binlog;
mod merge;
mod slots;

use std::{collections::BTreeMap, sync::Arc};

use crate::merge::MergeFn;
use parking_lot::RwLock;
use slots::Slots;

struct Dense<K, V> {
    slots: Slots<V>,
    index: BTreeMap<K, usize>, // the value in index must in slots' bounds
}
impl<K: Ord + Copy, V: Clone + Default> Dense<K, V> {
    fn with_capacity(cap: usize) -> Self {
        Dense {
            slots: Slots::with_capacity(cap),
            index: BTreeMap::new(),
        }
    }

    fn get(&self, key: &K) -> Option<V> {
        if let Some(index) = self.index.get(key) {
            unsafe { self.slots.read(*index) }
        } else {
            None
        }
    }

    fn set(&mut self, key: &K, value: V) -> Option<V> {
        if let Some(index) = self.index.get(key) {
            // already exists
            // swap the value
            unsafe { self.slots.swap(*index, value) }
        } else {
            // not exists
            let index = self.slots.create(value);
            self.index.insert(*key, index);
            None
        }
    }

    /// Nothing happens if the key does not exist.
    ///
    fn modify_with<F>(&self, key: &K, f: F)
    where
        F: FnOnce(Option<&mut V>),
    {
        if let Some(index) = self.index.get(key) {
            // exists
            unsafe { self.slots.modify_with(*index, f) };
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(index) = self.index.remove(key) {
            // exists
            unsafe { self.slots.take(index) }
        } else {
            // not exists
            None
        }
    }
}

/// A column that storage a set of same type data.
/// 
/// # Thread safe
/// 
/// Prop is designed to be thread-safe. Like any other database table,
/// prop has two types of lock, one is column lock(like table lock in mysql),
/// the other is cell lock(like row lock in mysql).
pub struct Prop<K, V, D = ()>
where
    K: Ord + Copy,
    V: Clone + Default,
{
    data: RwLock<Dense<K, V>>,
    merge_method: Arc<dyn MergeFn<V, D>>,
}

impl<K, V, D> Prop<K, V, D>
where
    K: Ord + Copy,
    V: Clone + Default,
{
    /// Create a new prop
    /// 
    /// a size of 4096 pre-allocation will be made.
    pub fn new() -> Self {
        Prop {
            data: RwLock::new(Dense::with_capacity(4096)),
            merge_method: Arc::new(|_old, _delta| false),
        }
    }

    /// Create a new prop, 
    /// with a merge function.
    pub fn with_merge(merge_method: impl MergeFn<V, D> + 'static) -> Self {
        Prop {
            data: RwLock::new(Dense::with_capacity(4096)),
            merge_method: Arc::new(merge_method),
        }
    }

    /// Get a value.
    /// 
    /// # Thread safe
    /// 
    /// - Column lock: read
    /// - Cell lock: read
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.read().get(key)
    }

    /// Set a value anyway.
    /// Create new cell if the key does not exist.
    ///
    /// May slower than modify_with
    /// because it needs to lock the whole map
    /// to create new cell if the key does not exist.
    /// 
    /// # Thread safe
    /// 
    /// - Column lock: write
    /// - Cell lock: write
    pub fn set(&self, key: &K, value: V) -> Option<V> {
        self.data.write().set(key, value)
    }

    /// Modify a cell by a function.
    /// 
    /// Nothing happen if key does not exist.
    /// 
    /// # Thread safe
    /// 
    /// - Column lock: read
    /// - Cell lock: write
    pub fn modify_with<F>(&self, key: &K, f: F)
    where
        F: FnOnce(Option<&mut V>),
    {
        self.data.read().modify_with(key, f);
    }

    /// Remove a value.
    /// 
    /// # Thread safe
    /// 
    /// - Column lock: write
    /// - Cell lock: write
    pub fn remove(&self, key: &K) -> Option<V> {
        self.data.write().remove(key)
    }

    /// Merge a delta value to a value.
    /// 
    /// # Thread safe
    /// 
    /// - Column lock: upgradable_read => write if key exists, read if key does not exist
    /// - Cell lock: write
    pub fn merge(&self, key: &K, delta: D) {
        let mut delete = false;
        let mut ctx = self.data.upgradable_read();
        ctx.modify_with(key, |old| {
            delete = (self.merge_method.as_ref())(old, delta);
        });
        if delete {
            ctx.with_upgraded(|x| x.remove(key));
        }
    }
}

mod test {

    #[test]
    fn test_set_get() {
        use super::Prop;

        let prop = Prop::<u64, i64>::new();
        prop.set(&1, 1);
        prop.set(&2, 2);
        prop.set(&3, 3);
        assert_eq!(prop.get(&1), Some(1));
        assert_eq!(prop.get(&2), Some(2));
        assert_eq!(prop.get(&3), Some(3));
        prop.set(&1, 2);
        assert_eq!(prop.get(&1), Some(2));

        assert_eq!(prop.get(&4), None);
    }

    #[test]
    fn test_modify_with() {
        use super::Prop;

        let mock_i64_add =
            |old: Option<&mut i64>| old.is_some().then(|| *old.unwrap() += 1).unwrap();

        let prop = Prop::<u64, i64>::new();
        prop.set(&1, 1);
        prop.modify_with(&1, mock_i64_add);
    }

    #[test]
    fn test_merge() {
        use super::Prop;

        let prop = Prop::<u64, i64>::new();
        prop.set(&1, 1);
        prop.merge(&1, ());
        assert_eq!(prop.get(&1), Some(1));

        let mock_merge = |old: Option<&mut i64>, delta: i32| {
            old.is_some().then(|| *old.unwrap() += delta.clone() as i64);
            false
        };

        let prop = Prop::<u64, i64, i32>::with_merge(mock_merge);
        prop.set(&1, 1);
        prop.merge(&1, 1);
        assert_eq!(prop.get(&1), Some(2));
        prop.merge(&1, 1);
        assert_eq!(prop.get(&1), Some(3));
        prop.merge(&1, 1000);
        assert_eq!(prop.get(&1), Some(1003));
    }
}
