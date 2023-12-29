use std::collections::BTreeSet;

use parking_lot::RwLock;

pub struct Cell<T>(RwLock<Option<T>>);
impl<T> Cell<T> {
    fn new() -> Self {
        Cell(RwLock::new(None))
    }

    fn is_valid(&self) -> bool {
        self.0.read().is_some()
    }
    fn set_value(&self, value: T) {
        self.0.write().replace(value);
    }
    fn set_none(&self) {
        *self.0.write() = None;
    }
    fn take_value(&self) -> Option<T> {
        self.0.write().take()
    }
    fn modify_with<F>(&self, f: F)
    where
        F: FnOnce(Option<&T>),
    {
        let mut ctx = self.0.write();
        f(ctx.as_ref());
    }
}

impl<T: Clone> Cell<T> {
    fn clone_value(&self) -> Option<T> {
        let ctx = self.0.read();
        ctx.as_ref().and_then(|x| Some(x.clone()))
    }
}
impl<T> Default for Cell<T> {
    fn default() -> Self {
        Cell::new()
    }
}

impl<T: Clone> Clone for Cell<T> {
    fn clone(&self) -> Self {
        Self(RwLock::new(self.0.read().clone()))
    }
}

pub(crate) struct Slots<T> {
    data: Vec<Cell<T>>,
    empty: BTreeSet<usize>,
}

impl<T: Clone + Default> Slots<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Slots {
            data: vec![Cell::new(); cap],
            empty: (0..cap).into_iter().collect(),
        }
    }
    pub fn new() -> Self {
        Slots::with_capacity(0)
    }

    /// Allocate n cells into slots.
    ///
    /// Property
    /// - (n)usize, the number of cells to allocate.
    ///
    /// Return
    /// - usize, the length of new slots.
    pub fn allocate(&mut self, n: usize) -> usize {
        (0..n).into_iter().into_iter().for_each(|_| {
            self.data.push(Cell::new());
            self.empty.insert(self.data.len() - 1);
        });
        self.data.len()
    }

    pub fn create(&mut self, value: T) -> usize {
        let index: usize;
        if let Some(i) = self.empty.pop_first() {
            index = i;
        } else {
            index = self.allocate(1) - 1;
        }
        let a = unsafe { self.data.get_unchecked(index) };
        a.set_value(value);
        index
    }

    /// # Safety
    /// index must inbound
    pub unsafe fn read(&self, index: usize) -> Option<T> {
        self.data.get_unchecked(index).clone_value()
    }

    /// # Safety
    /// index must inbound
    pub unsafe fn update(&mut self, index: usize, value: T) {
        self.data.get_unchecked(index).set_value(value);
    }

    /// # Safety
    /// index must inbound
    pub unsafe fn modify_with<F>(&self, index: usize, f: F)
    where F: FnOnce(Option<&T>) {
        self.data.get_unchecked(index).modify_with(f)
    }


    /// # Safety
    /// index must inbound
    pub unsafe fn swap(&mut self, index: usize, value: T) -> Option<T> {
        let old = self.data[index].clone_value();
        self.data.get_unchecked(index).set_value(value);
        old
    }

    /// # Safety
    /// index must inbound
    pub unsafe fn delete(&mut self, index: usize) {
        self.data.get_unchecked(index).set_none();
        self.empty.insert(index);
    }

    /// # Safety
    /// index must inbound
    pub unsafe fn take(&mut self, index: usize) -> Option<T> {
        self.data.get_unchecked(index).take_value()
    }
}
