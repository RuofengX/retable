//! A simple double lock implementation for Atomic protocol.
//!
//! The name is a bit misleading, since it just use a dense
//! vector to save datium but it allows the empty slot exists.
//!
//! Dense uses 2 levels of Read-Write lock to implement the atomic protocol.  
//! 
//! + First level lock is the 'table' lock, it protect the index, empty records
//! and the increase of inner data vector.  
//! + Second lock is the 'row' locks, they protect the inner cell inside the data vector.  
//!
//! To minimen the insertion time, the Dense allows empty slots, and use
//! a BTreeSet to tract all those empty slots. Whenever a value is deleted,
//! instead of drop the lock and the content, the slot would write-locked
//! and set to None, and then the index of this empty slot would pushed
//! into the 'empty' BTreeSet.
//!
//! When creating new value, the create function will first try to search
//! whether there is an empty slot, if so, reuse the empty slot,
//! gain the RwLock and then write the data.
//!

use parking_lot::RwLock;
use std::collections::BTreeSet;
use std::ops::DerefMut;
use std::{collections::BTreeMap, marker::PhantomData};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::protocol::{Atom, Atomic, LogWriter, MergeAssign};

struct DenseInner<K, V> {
    idx: BTreeMap<K, usize>,
    data: Vec<RwLock<Option<V>>>,
    empty: BTreeSet<usize>,
}
impl<K, V> Default for DenseInner<K, V> {
    fn default() -> Self {
        Self {
            idx: Default::default(),
            data: Default::default(),
            empty: Default::default(),
        }
    }
}

pub struct Dense<K, V, D> {
    inner: RwLock<DenseInner<K, V>>,
    _a: PhantomData<Atom<K, V, D>>,
}
impl<K, V, D> Default for Dense<K, V, D> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _a: Default::default(),
        }
    }
}

impl<K, V> Atomic for Dense<K, V, ()>
where
    K: Ord + Copy + AsBytes + FromBytes + FromZeroes,
    V: Default + Clone + AsBytes + FromBytes + FromZeroes,
{
    type K = K;

    type V = V;

    type D = ();

    unsafe fn create_unchecked(&self, key: &Self::K, value: &Self::V) {
        let mut inner = self.inner.upgradable_read();
        if inner.empty.is_empty() {
            inner.with_upgraded(|inner| {
                inner.data.push(RwLock::new(Some(value.clone())));
                inner.idx.insert(*key, inner.data.len() - 1);
            })
        } else {
            let empty_idx = inner.with_upgraded(|inner| inner.empty.pop_first().unwrap());
            *inner.data.get_unchecked(empty_idx).write() = Some(value.clone());
            inner.with_upgraded(|inner| {
                inner.idx.insert(*key, empty_idx);
            });
        }
    }

    unsafe fn read_unchecked(&self, key: &Self::K) -> Self::V {
        let inner = self.inner.read();
        let &idx = inner.idx.get(key).unwrap();
        let rtn = inner.data.get_unchecked(idx).read().clone().unwrap();
        rtn
    }

    unsafe fn update_unchecked(&self, key: &Self::K, value: &Self::V) -> Self::V {
        let inner = self.inner.read();
        let &idx = inner.idx.get(key).unwrap();
        let mut entry = inner.data.get_unchecked(idx).write();
        let mut buf = value.clone();
        std::mem::swap(entry.as_mut().unwrap(), &mut buf);
        buf
    }

    unsafe fn merge_unchecked(&self, key: &Self::K, delta: &Self::D) {
        let inner = self.inner.read();
        let &idx = inner.idx.get(key).unwrap();
        let mut entry = inner.data.get_unchecked(idx).write();
        entry.as_mut().unwrap().merge(delta.clone());
    }

    unsafe fn delete_unchecked(&self, key: &Self::K) -> Self::V {
        let mut inner = self.inner.upgradable_read();
        let &idx = inner.idx.get(key).unwrap();
        let mut buf: Option<V> = None;
        std::mem::swap(inner.data.get_unchecked(idx).write().deref_mut(), &mut buf);
        inner.with_upgraded(|inner| {
            inner.idx.remove(key);
            inner.empty.insert(idx);
        });
        buf.unwrap()
    }

    fn contains_key(&self, key: &Self::K) -> bool {
        let inner = self.inner.read();
        inner.idx.contains_key(key)
    }
    /// Get persistence handler
    #[cfg(feature = "persist")]
    fn get_persist(&self) -> &impl LogWriter<Self::K, Self::V, Self::D> {
        // TODO 设计一个带缓冲区的持久化日志磁盘记录器
        &()
    }
}
