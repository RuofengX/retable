use std::{path::PathBuf, sync::Arc, borrow::BorrowMut};

/// 一个双键索引的、原子化的、kv数据库
use kv::{self, Bincode, Store, Bucket};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::{
    atom::Atom,
    basic::{Delta, Value, EID},
    AtomStorage,
};

struct StatefulAtom {
    /// 反向索引
    pub index: EID,
    /// 实际数据
    pub data: Atom,
}

pub struct Database<'k>{
    db: Store,
    tables: FxHashMap<String, Bucket<'k, EID, Bincode<Value>>>,
    merge_fn: FxHashMap<String, Arc<dyn FnMut(EID, &Delta)->()>>,
}

impl <'k>Database<'k>{
    pub fn new(path: PathBuf) -> Result<Self, kv::Error> {
        Ok(Database {
            db: Store::new(kv::Config::new(path))?,
            tables: FxHashMap::default(),
            merge_fn: FxHashMap::default(),
        })
    }

    fn bucket(&mut self, prop: String) -> &mut Bucket<EID, Bincode<Value>>{
        let bucket = self.tables.entry(prop.clone()).or_insert(
            self.db.bucket(Some(&prop)).expect("Error when open bucket")
        );
        bucket

    }
}

impl <'k>AtomStorage for Database<'k>{
    fn set(&mut self, eid: EID, prop: &str, value: Value) -> () {
        let bucket = self.bucket(prop.into());
        bucket.set(&eid, &Bincode(value)).expect("Error when set");
    }

    fn get(&self, eid: EID, prop: &str) -> Option<Value> {
        let bucket = self.bucket(prop.into());
        let k = bucket.get(&eid).expect("Error when get atom");
        match k {
            Some(Bincode(v)) => Some(v),
            None => None,
        }
    }

    fn remove(&mut self, eid: EID, prop: &str) -> Option<()> {
        let bucket = self.bucket(prop.into());
        let r = bucket.remove(&eid).expect("Error when remove atom");
        match r {
            Some(_) => Some(()),
            None => None,
        }
    }

    fn register_merge<F>(&mut self, prop: &str, f: Arc<F>)
    where
        F: FnMut(EID, &Delta) -> (),
    {
        self.merge_fn.insert(prop.into(), f);
    }

    fn merge(&mut self, prop: &str, eid: EID, delta: &Delta) -> () {
        let bucket = self.bucket(prop.into());
    }
}
