mod binlog;

use parking_lot::RwLock;
use std::collections::BTreeMap;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

pub trait Key: Copy + Ord + Send {}
pub trait Value: AsBytes + FromBytes + FromZeroes + Clone + Send {}
impl<T> Key for T where T: Copy + Ord + Send {}
impl<T> Value for T where T: AsBytes + FromBytes + FromZeroes + Clone + Send {}

pub enum Command<K: Key, V: Value> {
    Create((K, V)),
    Update((K, V)),
    Read(K),
    Take(K),
    Delete(K),

    Merge((K, V)),
    CompareAndSwap((K, V, V)), // Key, Old, New
}

pub trait MergeFn<K: Key, V: Value>: Fn(&K, Option<V>, V) -> Option<V> {}
impl<T, K: Key, V: Value> MergeFn<K, V> for T where T: Fn(&K, Option<V>, V) -> Option<V> {}

pub struct RawProp<K: Key, V: Value, M: MergeFn<K, V>> {
    index: BTreeMap<K, usize>,
    data: Vec<Option<V>>,
    empty_slot: Vec<usize>,
    merge_op: M,
}

/// CRUD api
impl<K: Key, V: Value, M: MergeFn<K, V>> RawProp<K, V, M> {
    pub fn create(&mut self, k: &K, v: V) {
        if self.contains_key(&k) {
            unsafe { self.update_uncheck(k, v) };
        } else {
            unsafe { self.create_uncheck(k, v) }
        }
    }

    pub fn read(&self, k: &K) -> Option<V> {
        if let Some(&index) = self.index.get(k) {
            unsafe { self.read_uncheck(index).clone() }
        } else {
            None
        }
    }

    pub fn update(&mut self, k: &K, v: V) -> Option<V> {
        if !self.contains_key(&k) {
            unsafe { self.create_uncheck(k, v) };
            None
        } else {
            Some(unsafe { self.update_uncheck(k, v) })
        }
    }

    pub fn delete(&mut self, k: &K) -> Option<V> {
        if let Some(index) = self.index.remove(k) {
            self.empty_slot.push(index);
            unsafe { self.data.get_unchecked(index) }.clone()
        } else {
            None
        }
    }
}

impl<K: Key, V: Value, M: MergeFn<K, V>> RawProp<K, V, M> {
    pub fn new(merge_op: M) -> RawProp<K, V, M> {
        RawProp::<K, V, M> {
            index: BTreeMap::new(),
            data: Vec::new(),
            empty_slot: Vec::new(),
            merge_op,
        }
    }

    /// Merge one value into self, return the old value.
    pub fn merge(&mut self, key: &K, delta: V) -> Option<V> {
        let old = self.read(key);
        let new = (self.merge_op)(key, old, delta);
        if let Some(new) = new{
            self.update(key, new)
        } else {
            self.delete(key)
        }
    }
}

/// Interact with self
impl<K: Key, V: Value, M: MergeFn<K, V>> RawProp<K, V, M> {
    pub fn contains_key(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }
}

/// Private method
impl<K: Key, V: Value, M: MergeFn<K, V>> RawProp<K, V, M> {
    /// # Panic
    /// Panic when key not exists.
    #[inline]
    fn get_index_unchecked(&self, k: &K) -> usize {
        self.index.get(k).unwrap().clone()
    }

    /// Safety: entry must valid.
    #[inline]
    unsafe fn get_value_unchecked(&self, index: usize) -> &Option<V> {
        self.data.get_unchecked(index)
    }

    /// Safety: k must not exist before.
    #[inline]
    pub unsafe fn create_uncheck(&mut self, k: &K, v: V) {
        let len = self.data.len();
        self.data.push(Some(v));
        self.index.insert(*k, len);
    }

    /// Safety: k must exist before.
    #[inline]
    pub unsafe fn read_uncheck(&self, index: usize) -> &Option<V> {
        self.get_value_unchecked(index)
    }

    /// Safety: k must exist before.
    #[inline]
    pub unsafe fn update_uncheck(&mut self, k: &K, v: V) -> V {
        let index = self.get_index_unchecked(k);
        let entry = self.data.get_unchecked_mut(index);
        let old = entry.as_ref().unwrap().clone();
        *entry = Some(v);
        old
    }
}

pub struct Prop<K: Key, V: Value, M: MergeFn<K, V>> {
    inner: RwLock<RawProp<K, V, M>>,
}
impl<K: Key, V: Value, M: MergeFn<K, V>> Prop<K, V, M> {
    pub fn new(merge_op: M) -> Prop<K, V, M> {
        Self {
            inner: RwLock::new(RawProp::<K, V, M> {
                index: BTreeMap::new(),
                data: Vec::new(),
                empty_slot: Vec::new(),
                merge_op,
            }),
        }
    }
}

/// Thread safe CRUD api
impl<K: Key, V: Value, M: MergeFn<K, V>> Prop<K, V, M> {
    pub fn create(&self, k: &K, v: V) {
        let mut wtx = self.inner.write();
        wtx.create(k, v)
    }

    pub fn read(&self, k: &K) -> Option<V> {
        let rtx = self.inner.read();
        rtx.read(k)
    }

    pub fn update(&mut self, k: &K, v: V) -> Option<V> {
        let mut wtx = self.inner.write();
        wtx.update(k, v)
    }

    pub fn delete(&mut self, k: &K) -> Option<V> {
        let mut wtx = self.inner.write();
        wtx.delete(k)
    }
}
