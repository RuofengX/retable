use std::collections::{BTreeMap, BTreeSet};

use parking_lot::RwLock;

mod binlog;

struct Column<T: Default + Clone> {
    slot: Vec<RwLock<T>>,
    invalid: BTreeSet<usize>,
}
impl<T: Default + Clone> Column<T> {
    fn new() -> Self {
        Self::from(vec![])
    }

    fn from(raw: Vec<T>) -> Self {
        let len = raw.len();
        Self {
            slot: raw.into_iter().map(|x| RwLock::new(x)).collect(),
            invalid: BTreeSet::new(),
        }
    }

    fn pre_allocate(&mut self, n: usize) {
        let len = self.slot.len();
        (len..(len + n)).for_each(|x| {
            self.slot.push(RwLock::new(T::default()));
            self.invalid.insert(x);
        })
    }

    fn extend_until(&mut self, k: usize) {
        let len = self.slot.len();
        let x = k + 1 - len;
        if x > 0 {
            self.pre_allocate(x)
        }
    }

    fn contains_key(&self, k: usize) -> bool {
        if k >= self.slot.len() {
            return false;
        }
        if self.invalid.contains(&k) {
            return false;
        }
        true
    }

    fn create(&mut self, v: T) -> usize {
        if self.invalid.is_empty() {
            self.pre_allocate(1);
        }
        let index = self.invalid.pop_first().unwrap();
        let mut entry = unsafe { self.slot.get_unchecked(index).write() };
        *entry = v;
        index
    }

    fn read(&self, k: usize) -> Option<T> {
        if self.invalid.contains(&k) {
            return None;
        }
        if let Some(value) = self.slot.get(k) {
            Some(value.read().clone())
        } else {
            None
        }
    }

    fn update_no_extend(&mut self, k: usize, v: T) -> Option<T> {
        if k >= self.slot.len() {
            return None;
        }
        if self.invalid.contains(&k) {
            return None;
        }
        let mut entry = unsafe { self.slot.get_unchecked_mut(k).upgradable_read() };
        let old = entry.clone();
        entry.with_upgraded(|x| *x = v);
        Some(old)
    }

    fn update(&mut self, k: usize, v: T) -> Option<T> {
        self.extend_until(k);
        self.update(k, v)
    }

    fn delete(&mut self, k: usize) -> Option<T> {
        if k >= self.slot.len() {
            return None;
        }
        if self.invalid.contains(&k) {
            return None;
        }
        let entry = unsafe { self.slot.get_unchecked_mut(k).read() };
        let old = entry.clone();
        self.invalid.insert(k);
        Some(old)
    }
}
