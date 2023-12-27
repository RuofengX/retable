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

pub struct RawProp<K: Key, V: Value> {
    index: BTreeMap<K, usize>,
    data: Vec<Option<V>>,
    empty_slot: Vec<usize>,
}

/// CRUD api
impl<K: Key, V: Value> RawProp<K, V> {
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

/// Interact with self
impl<K: Key, V: Value> RawProp<K, V> {
    pub fn contains_key(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }
}

/// Private method
impl<K: Key, V: Value> RawProp<K, V> {
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

pub struct Prop<K: Key, V: Value> {
    inner: RwLock<RawProp<K, V>>,
}
impl<K: Key, V: Value> Prop<K, V> {
    pub fn new() -> Prop<K, V> {
        Self {
            inner: RwLock::new(RawProp::<K, V> {
                index: BTreeMap::new(),
                data: Vec::new(),
                empty_slot: Vec::new(),
            }),
        }
    }
}

/// Thread safe CRUD api
impl<K: Key, V: Value> Prop<K, V> {
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
