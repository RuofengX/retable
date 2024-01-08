use std::{collections::BTreeMap, marker::PhantomData};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::protocol::{Atom, Atomic};

pub struct Dense<K, V, D> {
    idx: BTreeMap<K, usize>,
    data: Vec<Option<V>>,
    _a: PhantomData<Atom<K, V, D>>,
}
impl<K, V, D> Default for Dense<K, V, D> {
    fn default() -> Self {
        Self {
            idx: Default::default(),
            data: Default::default(),
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
        todo!()
    }

    unsafe fn read_unchecked(&self, key: &Self::K) -> Self::V {
        todo!()
    }

    unsafe fn update_unchecked(&self, key: &Self::K, value: &Self::V) -> Self::V {
        todo!()
    }

    unsafe fn merge_unchecked(&self, key: &Self::K, delta: &Self::D) {
        todo!()
    }

    unsafe fn delete_unchecked(&self, key: &Self::K) -> Self::V {
        todo!()
    }

    fn contains_key(&self, key: &Self::K) -> bool {
        todo!()
    }

    fn get_persist(&self) -> &() {
        todo!()
    }
}
