use std::collections::{BTreeMap, BTreeSet, HashSet};

use parking_lot::RwLock;
use slab::Slab;

mod binlog;
mod slot;


pub struct Dense<K: Copy + Ord, V: Default + Clone> {
    index: BTreeMap<K, usize>,
    data: Slab<Cell<V>>,
}

pub struct Column<K: Copy + Ord, V: Default + Clone> (RwLock<Dense<K,V>>);

impl<K: Copy + Ord, V: Default + Clone> Column<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(RwLock::new(
            Dense{
                index: BTreeMap::new(),
                data: Slab::with_capacity(capacity),
            }
        ))
    }

    pub fn create(&self, key: &K, value: V){
        let table_lock = self.0.upgradable_read();
        if table_lock.data.
    }


    pub fn get(&self, key: &K) -> Option<K> {
        if let Some(index) = self.valid.get(key) {
            let entry = unsafe { self.slot.get_unchecked(*index).read() };
            Some(entry.as_ref().unwrap().clone())
        } else {
            return None;
        }
    }

    pub fn set(&mut self, key: &K, value: K) -> Option<K> {
        if let Some(index) = self.valid.get(key) {
            self.update_exist(*index, value)
        } else {
            let index = self.create_any(value);
            self.valid.insert(*key, index);
            None
        }
    }
    // TODO: Add gc, iter, merge
}

/// Base function
impl<K: Copy + Ord, T: Default + Clone> Column<K, T> {
    fn create_any(&mut self, v: T) -> usize {
        if let Some(index) = self.empty.pop_first() {
            let mut entry = unsafe { self.slot.get_unchecked_mut(index).write() };
            *entry = Some(v);
            index
        } else {
            self.slot.push(RwLock::new(Some(v)));
            self.slot.len()
        }
    }

    fn allocate_empty(&mut self) -> usize {
        self.slot.push(RwLock::new(None));
        self.slot.len()
    }

    fn update_exist(&self, index: usize, v: T) -> Option<T> {
        let mut entry = unsafe { self.slot.get_unchecked(index).upgradable_read() };
        let old = entry.clone();
        entry.with_upgraded(|x| *x = Some(v));
        old
    }

    fn delete_exist(&self, index: usize) -> Option<T> {
        let mut entry = unsafe { self.slot.get_unchecked(index).upgradable_read() };
        let old = entry.clone();
        entry.with_upgraded(|x| *x = None);
        old
    }
}
