use crate::{Error, MergeFn, MergeFnClosure};
use kv::{self, Bincode, Bucket, Store};
use rustc_hash::FxHashMap;
use sled::transaction::ConflictableTransactionError;
/// 一个双键索引的、原子化的、kv数据库
use std::{path::PathBuf, sync::Arc};

use crate::{
    basic::{Delta, Value, EID},
    AtomStorage,
};

pub struct Database {
    db: Store,
    merge_fn: FxHashMap<String, MergeFnClosure>, // FxHashMap
} 

impl Database {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        Ok(Database {
            db: Store::new(kv::Config::new(path))?,
            merge_fn: FxHashMap::default(),
        })
    }

    /// Get bucket ref.
    fn bucket(&self, prop: String) -> Bucket<'static, EID, Bincode<Value>> {
        let bucket = self.db.bucket(Some(&prop)).expect("Error when get bucket");
        bucket
    }
}

impl AtomStorage for Database {
    fn get(&self, eid: EID, prop: &str) -> Option<Value> {
        let bucket = self.bucket(prop.into());
        let k = bucket.get(&eid).expect("Error when get atom");
        match k {
            Some(Bincode(v)) => Some(v),
            None => None,
        }
    }

    fn set(&self, eid: EID, prop: &str, value: Value, retrieve: bool) -> Option<Value> {
        let bucket = self.bucket(prop.into());
        if let Some(Bincode(v)) = bucket.set(&eid, &Bincode(value)).expect("Error when set") {
            if retrieve {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn remove(&self, eid: EID, prop: &str, retrieve: bool) -> Option<Value> {
        let bucket = self.bucket(prop.into());
        if let Some(Bincode(v)) = bucket.remove(&eid).expect("Error when set") {
            if retrieve {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn register_merge(&mut self, prop: &str, f: Arc<dyn MergeFn>) {
        self.merge_fn.insert(prop.into(), f);
    }

    fn merge(&self, prop: &str, eid: EID, delta: Delta) -> () {
        let bucket = self.bucket(prop.into());
        if let Some(f) = self.merge_fn.get(prop) {
            let f = f.clone();
            let _ = bucket.transaction(
                |trans|{
                    if let Ok(Some(Bincode(mut value))) = trans.get(&eid){
                        f(&mut value, &delta);
                        let _ = trans.set(&eid, &Bincode(value));
                    }
                    Ok::<(), ConflictableTransactionError<Error>>(())
                }
            );
            // FIXME: 使用事务进行原子化操作
        } 
        ()
    }
}

// mod test {
//     #![allow(unused_imports)]
//     use super::*;
//     #[test]
//     fn test_set_get() {
//         let db = Database::new("db".into()).unwrap();
//         db.set(EID(1), "name", Value::String("tom".into()), false);
//         assert_eq!(db.get(EID(1), "name"), Some(Value::String("tom".into())));
//     }
//     fn test_merge() {
//         let db = Database::new("test.db".into()).unwrap();
//         db.register_merge("name", Arc::new(|bucket, eid, delta| {
//             if let Some(Value::String(s)) = delta.value {
//                 bucket.set(eid, &Bincode(Value::String(format!("{}{}", s, "1"))));
//             }
//         }
//     }
// }
