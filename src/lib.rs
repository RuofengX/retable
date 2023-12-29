mod slots;

use std::collections::BTreeMap;

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
    fn modify_with<F>(&self, key: &K, f: F)
    where
        F: FnOnce(Option<&V>),
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

pub struct Prop<K, V>
where
    K: Ord + Copy,
    V: Clone + Default,
{
    data: RwLock<Dense<K, V>>,
}

impl<K, V> Prop<K, V>
where
    K: Ord + Copy,
    V: Clone + Default,
{
    pub fn new() -> Self {
        Prop {
            data: RwLock::new(Dense::with_capacity(4096)),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.data.read().get(key)
    }

    /// Set a value anyway.
    /// Create new cell if the key does not exist.
    ///
    /// May slower than modify_with
    /// because it needs to lock the whole map
    /// to create new cell if the key does not exist.
    pub fn set(&self, key: K, value: V) -> Option<V> {
        self.data.write().set(&key, value)
    }

    pub fn modify_with<F>(&self, key: K, f: F)
    where
        F: FnOnce(Option<&V>),
    {
        self.data.read().modify_with(&key, f)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.data.write().remove(key)
    }
}
