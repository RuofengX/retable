use std::{marker::PhantomData, path::Path};
use zerocopy::{AsBytes, FromBytes};

use lsm_tree::{Config, Tree};

use crate::protocol::{LogWriter, MergeAssign, LogReader};

pub struct LsmPersist<K, V, D> {
    inner: Tree,
    _a: PhantomData<(K, V, D)>,
}

impl<K, V, D> LsmPersist<K, V, D> {
    pub fn new(folder_path: &Path) -> Self {
        let name = std::any::type_name::<(K, V, D)>();
        LsmPersist {
            inner: Config::new(folder_path.join(name))
                .flush_threads(4)
                .block_size(256 * 1024 * 1024)
                .open()
                .unwrap(),
            _a: PhantomData,
        }
    }
}

impl<K, V, D> LogWriter<K, V, D> for LsmPersist<K, V, D>
where
    K: FromBytes + AsBytes,
    V: FromBytes + AsBytes + MergeAssign<Delta = D>,
    D: FromBytes + AsBytes,
{
    fn create(&self, key: &K, value: &V) {
        self.inner.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }

    fn update(&self, key: &K, value: &V) {
        self.inner.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }

    fn merge(&self, key: &K, delta: &D) {
        let mut old = V::read_from(&self.inner.get(key.as_bytes()).unwrap().unwrap()).unwrap();
        old.merge(D::read_from(delta.as_bytes()).unwrap());
        self.inner.insert(key.as_bytes(), old.as_bytes()).unwrap();
    }

    fn delete(&self, key: &K) {
        self.inner.remove(key.as_bytes()).unwrap();
    }
}

impl<K,V,D>Iterator for LsmPersist<K,V,D> {
    type Item = (K,V);

    fn next(&mut self) -> Option<Self::Item> {
        todo!();
        let a = self.inner.iter();
        let b = a.next();
    }
}
impl <K,V,D> LogReader<K,V,> for LsmPersist<K,V,D>{
}