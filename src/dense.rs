use parking_lot::RwLock;
use std::collections::BTreeSet;
use std::ops::DerefMut;
use std::{collections::BTreeMap, marker::PhantomData};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::protocol::{LogWriter, MergeAssign};
use crate::protocol::{Atom, Atomic};

struct DenseInner<K, V> {
    idx: BTreeMap<K, usize>,
    data: Vec<RwLock<Option<V>>>, // TODO: 使用RwLock重写，减少表写锁
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
        let mut inner = self.inner.write();
        if let Some(empty_idx) = inner.empty.pop_first() {
            let empty_entry = inner.data.get_unchecked_mut(empty_idx);
            *empty_entry = Some(value.clone());
        } else {
            inner.data.push(Some(value.clone()));
            inner.idx.insert(*key, inner.data.len() - 1);
        }
    }

    unsafe fn read_unchecked(&self, key: &Self::K) -> Self::V {
        let inner = self.inner.read();
        let &idx = inner.idx.get(key).unwrap();
        inner.data.get_unchecked(idx).unwrap()
    }

    unsafe fn update_unchecked(&self, key: &Self::K, value: &Self::V) -> Self::V {
        let mut inner = self.inner.write();
        let &idx = inner.idx.get(key).unwrap();
        let entry = inner.data.get_unchecked_mut(idx).as_mut().unwrap();
        let mut old = Self::V::new_zeroed();
        std::mem::swap(entry, &mut old);
        old
    }

    unsafe fn merge_unchecked(&self, key: &Self::K, delta: &Self::D) {
        let mut inner = self.inner.write();
        let &idx = inner.idx.get(key).unwrap();
        let entry = inner.data.get_unchecked_mut(idx).as_mut().unwrap();
        entry.merge(delta.clone());
    }

    unsafe fn delete_unchecked(&self, key: &Self::K) -> Self::V {
        todo!()
    }

    fn contains_key(&self, key: &Self::K) -> bool {
        let inner = self.inner.read();
        inner.idx.contains_key(key)
    }
    /// Get persistence handler
    #[cfg(feature = "persist")]
    fn get_persist(&self) -> &impl LogWriter<Self::K, Self::V, Self::D> {
        todo!()
    }
}
