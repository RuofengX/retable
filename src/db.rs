use std::{marker::PhantomData, path::Path};

use crate::{
    error::Error,
    atom::{Delta, Key, Value},
    tree::Tree,
};

pub struct Database<K> {
    _k: PhantomData<K>,
}

impl<K> Database<K>
where
    K: Key,
{
    pub fn new() -> Self {
        todo!()
    }

    pub fn open(path: &Path) -> Self {
        todo!()
    }

    pub fn flush(&self) -> Result<(), Error> {
        todo!()
    }

    pub fn open_tree<V, D>(&self, k: K) -> Result<Tree<K, V, D>, Error>
    where
        V: Value,
        D: Delta,
    {
        todo!()
    }
}
