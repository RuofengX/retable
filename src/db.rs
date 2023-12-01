/// 一个双键索引的、原子化的、kv数据库
use crate::{Error, MergeFn, MergeFnClosure};
use kv::{self, Bincode, Bucket, Store, TransactionError};
use std::{collections::BTreeMap, sync::Arc};

use crate::basic::{Delta, Value, EID};

pub use kv::Config;

/// prop存储方案必须要实现的特质
/// 对单一属性的存储方案的签名
pub trait AtomStorage {
    /// 获取eid的属性
    fn get(&self, eid: EID, prop: &'static str) -> Option<Value>;

    /// 为eid的prop属性设置一个数值
    /// 如不存在则生成新的
    fn set(&mut self, eid: EID, prop: &'static str, value: Value, retrieve: bool) -> Option<Value>;

    /// 删除eid的属性
    /// kv实现内部可变性
    fn remove(&self, eid: EID, prop: &'static str, retrieve: bool) -> Option<Value>;

    /// 注册merge函数
    fn register_merge(&mut self, prop: &'static str, f: Arc<MergeFn>);

    /// 使用merge函数合并属性，
    /// 为最大化性能抛弃所有结果
    fn merge(&self, prop: &'static str, eid: EID, delta: Delta) -> ();

    // TODO: 添加批量merge操作
    // TODO: 添加输入、输出流
    // TODO: 添加默认的merge函数
}

pub struct Database {
    db: Store,
    merge_fn: BTreeMap<&'static str, MergeFnClosure>, // FxHashMap
    buckets: BTreeMap<&'static str, Bucket<'static, EID, Bincode<Value>>>,
    //TODO: ADD LRU CACHE
}

impl Database {
    pub fn new(conf: Config) -> Result<Self, Error> {
        Ok(Database {
            db: Store::new(conf)?,
            merge_fn: BTreeMap::default(),
            buckets: BTreeMap::default(),
        })
    }
    /// Get bucket ref. Dont create new
    fn bucket<'s>(
        &'s self,
        prop: &'static str,
    ) -> Option<&'s Bucket<'static, EID, Bincode<Value>>> {
        self.buckets.get(prop)
    }

    /// Get bucket ref, create if non-exist.
    fn bucket_mut<'s>(
        &'s mut self,
        prop: &'static str,
    ) -> &'s Bucket<'static, EID, Bincode<Value>> {
        let bucket = self.buckets.entry(prop).or_insert(
            self.db
                .bucket(Some(prop))
                .expect("Error when create bucket"),
        );
        bucket
    }
}
impl Default for Database {
    fn default() -> Database {
        Database {
            db: Store::new(kv::Config::new("db/default")).expect("Error when create database."),
            merge_fn: BTreeMap::default(),
            buckets: BTreeMap::default(),
        }
    }
}

impl AtomStorage for Database {
    fn get(&self, eid: EID, prop: &'static str) -> Option<Value> {
        if let Some(bucket) = self.bucket(prop) {
            let k = bucket.get(&eid).expect("Error when get atom");
            match k {
                Some(Bincode(v)) => Some(v),
                None => None,
            }
        } else {
            None
        }
    }

    fn set(&mut self, eid: EID, prop: &'static str, value: Value, retrieve: bool) -> Option<Value> {
        let bucket = self.bucket_mut(prop);
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

    fn remove(&self, eid: EID, prop: &'static str, retrieve: bool) -> Option<Value> {
        if let Some(bucket) = self.bucket(prop) {
            if let Some(Bincode(v)) = bucket.remove(&eid).expect("Error when remove prop") {
                if retrieve {
                    Some(v)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn register_merge(&mut self, prop: &'static str, f: Arc<MergeFn>) {
        self.merge_fn.insert(prop, f);
    }

    fn merge(&self, prop: &'static str, eid: EID, delta: Delta) -> () {
        if let Some(bucket) = self.bucket(prop) {
            if let Some(f) = self.merge_fn.get(prop) {
                let _ = bucket.transaction(|trans| {
                    let value = trans.get(&eid).expect("Error when get value.");
                    if let Some(Bincode(mut value)) = value {
                        f(&mut value, &delta);
                        let _ = trans.set(&eid, &Bincode(value));
                    } else {
                        let _ = trans.set(&eid, &Bincode(delta));
                    }
                    Ok::<(), TransactionError<Error>>(())
                });
            }
        };
        ()
    }
}

mod test {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use std::thread;
    #[test]
    fn test_merge() {
        let eid = EID::new(1);
        let prop = "prop1";

        let int_merge = |value: &mut Value, delta: &Delta| {
            if let (Value::Int(v), Value::Int(d)) = (value, delta) {
                *v += d;
            }
        };
        let int_merge = Arc::new(int_merge);

        let mut db = Database::new(Config::new("db/test/merge").temporary(true)).unwrap();
        db.register_merge(prop, int_merge);

        db.bucket_mut(prop);
        db.merge(prop, eid, Delta::Int(1));
        assert_eq!(db.get(eid, prop), Some(Value::Int(1)));
        db.merge(prop, eid, Delta::Int(1));
        assert_eq!(db.get(eid, prop), Some(Value::Int(2)));

        // 多线程同时merge
        let db = Arc::new(db);
        let mut jh = Vec::new();
        for _ in 0..64 {
            let db_dup = db.clone();
            jh.push(thread::spawn(move || {
                for _ in 0..1000 {
                    db_dup.merge(prop, eid, Delta::Int(1));
                }
            }));
            //TODO: 让Database可多线程访问
        }
        for i in jh {
            i.join().unwrap();
        }
        assert_eq!(db.get(eid, prop), Some(Value::Int(64002)));
    }
}
