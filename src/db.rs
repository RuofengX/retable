use std::path::PathBuf;

/// 一个双键索引的、原子化的、kv数据库
use kv::{Store, self};
use parking_lot::RwLock;

use crate::{atom::Atom, basic::{EID, Value, Delta}, AtomStorage};
 
struct StatefulAtom{
    /// 反向索引
    pub index: EID,
    /// 实际数据
    pub data: Atom,
}

pub struct Database {
    db: Store, 
}

impl Database{
    pub fn new(path: PathBuf) -> Result<Self, kv::Error>{
        Ok(
            Database {
                db: Store::new(kv::Config::new(path))?
            }
        )
    }
}
impl AtomStorage for Database{
    fn append(&mut self, eid: EID, prop:&str, value: Value) -> () {
        todo!()
    }

    fn get(&self, eid: EID, prop: &str) -> Option<&RwLock<Value>> {
        todo!()
    }

    fn remove(&mut self, eid: EID, prop: &str) -> Option<()> {
        todo!()
    }

    fn tick<F>(&mut self, prop: &str, f: &mut F)
    where
        F: FnMut(&mut Value) -> () {
        todo!()
    }

    fn register_merge<F>(&mut self, prop: &str, f: F)
    where
        F: FnMut(&EID, &Delta) -> () {
        todo!()
    }

    fn merge(&mut self, prop: &str, eid: EID, delta: &Delta) -> () {
        todo!()
    }
}

