use std::sync::Arc;

use crate::atom::{Atom, Delta, Key, Value};

pub trait TickFn<K, V, D>: Fn(usize, V, &Tree<K, V, D>) -> Option<D> + Send + Sync + 'static
where
    K: Key,
    V: Value,
    D: Delta,
{
}
pub trait MergeFn<K, V, D>: Fn(usize, Option<V>, D) -> Option<V> + Send + Sync + 'static
where
    K: Key,
    V: Value,
    D: Delta,
{
}

pub struct Tree<K, V, D>
where
    K: Key,
    V: Value,
    D: Delta,
{
    tick: Arc<dyn TickFn<K, V, D>>,
    merge: Arc<dyn MergeFn<K, V, D>>,
    data: Vec<Atom<K, V>>,
}
impl<K, V, D> Tree<K, V, D>
where
    K: Key,
    V: Value,
    D: Delta,
{
    pub fn new(tick: impl TickFn<K, V, D>, merge: impl MergeFn<K, V, D>) -> Tree<K, V, D> {
        Tree::<K, V, D> {
            tick: Arc::new(tick),
            merge: Arc::new(merge),
            data: Vec::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<Atom<K, V>> {
        todo!()
    }

    pub fn update(&self, key: &K, value: V) -> () {
        todo!()
    }

    pub fn set(&self, key: &K, value: V) -> Option<Atom<K, V>> {
        todo!()
    }

    pub fn remove(&self, key: &K) -> () {
        todo!()
    }

    pub fn take(&self, key: &K) -> Option<Atom<K, V>> {
        todo!()
    }

    pub fn merge(&self, key: &K, delta: D) -> () {
        todo!()
    }
}
