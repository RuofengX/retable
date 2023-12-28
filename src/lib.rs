use std::collections::{BTreeMap, BTreeSet, HashSet};

use parking_lot::RwLock;
use slab::Slab;

mod binlog;

pub struct Column<K: Copy + Ord, T: Default + Clone> {
    valid: BTreeMap<K, usize>,
    empty: BTreeSet<usize>,
    slot: Vec<RwLock<Option<T>>>,
}

impl<K: Copy + Ord, T: Default + Clone> Column<K, T> {
    pub fn new() -> Self {
        Self {
            valid: BTreeMap::new(),
            empty: BTreeSet::new(),
            slot: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut slot = Vec::new();
        let mut empty = BTreeSet::new();
        for ele in (0..capacity).into_iter() {
            slot.push(RwLock::new(None));
            empty.insert(ele);
        }
        Self {
            valid: BTreeMap::new(),
            empty,
            slot,
        }
    }

    pub fn pre_allocate(&mut self, size: usize) {
        (0..size).into_iter().for_each(|_| {
            let index = self.allocate_empty();
            self.empty.insert(index);
        })
    }

    pub fn get(&self, key: &K) -> Option<T> {
        if let Some(index) = self.valid.get(key) {
            let entry = unsafe { self.slot.get_unchecked(*index).read() };
            Some(entry.as_ref().unwrap().clone())
        } else {
            return None;
        }
    }

    pub fn set(&mut self, key: &K, value: T) -> Option<T> {
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
