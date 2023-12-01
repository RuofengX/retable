/// 一个双键索引的、原子化的、kv数据库
use crate::{Error, MergeFn, PropBucket};
use std::collections::BTreeMap;

use crate::basic::{Delta, Value, EID};

use sled::{Config, Db};

/// prop存储方案必须要实现的特质
/// 对单一属性的存储方案的签名
pub trait AtomStorage: Sync + Send {
    /// 获取eid的属性
    fn get(&self, eid: EID, prop: &'static str) -> Option<Value>;

    /// 为eid的prop属性设置一个数值
    fn set(&self, eid: EID, prop: &'static str, value: Value, retrieve: bool) -> Option<Value>;

    /// 为eid的prop属性设置一个数值
    /// 如不存在则生成新的
    fn set_mut(
        &mut self,
        eid: EID,
        prop: &'static str,
        value: Value,
        retrieve: bool,
    ) -> Option<Value>;

    /// 删除eid的属性
    /// kv实现内部可变性
    fn remove(&self, eid: EID, prop: &'static str, retrieve: bool) -> Option<Value>;

    /// 注册merge函数
    fn register_merge(&self, prop: &'static str, f: MergeFn) -> Result<(), Error>;

    /// 使用merge函数合并属性，
    /// 为最大化性能抛弃所有结果
    fn merge(&self, prop: &'static str, eid: EID, delta: Delta) -> ();

    /// 获取单一属性的Bucket，不存在则返回None
    fn bucket(&self, prop: &'static str) -> Option<&PropBucket>;

    /// 获取单一属性的Bucket，如不存在则创建
    fn bucket_create(&mut self, prop: &'static str) -> &PropBucket;

    // TODO: 添加批量merge操作
    // TODO: 添加输入、输出流
    // TODO: 添加默认的merge函数
}

pub struct Database {
    db: Db,
    buckets: BTreeMap<&'static str, crate::PropBucket>,
    //TODO: 添加LRU缓存
}

impl Database {
    pub fn new(conf: Config) -> Result<Self, Error> {
        Ok(Database {
            db: conf.open()?,
            buckets: BTreeMap::default(),
        })
    }
}

impl Default for Database {
    fn default() -> Database {
        Database {
            db: Config::default()
                .path("db/default")
                .cache_capacity(1_000_000_000)
                .flush_every_ms(Some(1000))
                .open()
                .expect("Error when open default db"),
            buckets: BTreeMap::default(),
        }
    }
}

impl AtomStorage for Database {
    fn get(&self, eid: EID, prop: &'static str) -> Option<Value> {
        if let Some(bucket) = self.bucket(prop) {
            let k = bucket.get(&eid).expect("Error when get atom");
            match k {
                Some(v) => Some(v),
                None => None,
            }
        } else {
            None
        }
    }

    fn set(&self, eid: EID, prop: &'static str, value: Value, retrieve: bool) -> Option<Value> {
        if let Some(bucket) = self.bucket(prop) {
            if let Some(v) = bucket.insert(&eid, &value).expect("Error when set") {
                if retrieve {
                    return Some(v);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        };
        None
    }

    fn set_mut(
        &mut self,
        eid: EID,
        prop: &'static str,
        value: Value,
        retrieve: bool,
    ) -> Option<Value> {
        let bucket = self.bucket_create(prop);
        if let Some(v) = bucket.insert(&eid, &value).expect("Error when set") {
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
            if let Some(v) = bucket.remove(&eid).expect("Error when remove prop") {
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

    fn register_merge(&self, prop: &'static str, f: MergeFn) -> Result<(), Error>{
        if let Some(bucket) = self.bucket(prop) {
            bucket.set_merge_operator(f);
            Ok(())
        } else {
            Err(Error::PropError(prop.to_string()))
        }
    }

    fn merge(&self, prop: &'static str, eid: EID, delta: Delta) -> () {
        let bucket = self.bucket(prop).expect("尚未注册merge函数");
        bucket.merge(&eid, &delta).unwrap();
    }

    fn bucket<'s>(&'s self, prop: &'static str) -> Option<&'s PropBucket> {
        self.buckets.get(prop)
    }

    fn bucket_create<'s>(&'s mut self, prop: &'static str) -> &'s PropBucket {
        let bucket = self
            .buckets
            .entry(prop)
            .or_insert(typed_sled::Tree::<EID, Value>::open(&self.db, prop));
        bucket
    }
}

mod test {

    #[allow(unused_imports)]
    use crate::MergeFn;
    #[allow(unused_imports)]
    use std::sync::Arc;

    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use std::thread;
    #[test]
    fn test_merge() {
        let eid = EID::new(1);
        let prop = "prop1";

        fn int_merge(_eid: EID, value: Option<Value>, delta: Delta) -> Option<Value> {
            if value.is_none() {
                return Some(delta);
            }
            if let Some(Value::Int(mut v)) = value {
                if let Value::Int(d) = delta {
                    v += d;
                    return Some(Value::Int(v));
                }
            }
            None
        }
        static INT_MERGE_FN: fn(EID, Option<Value>, Delta) -> Option<Value> = int_merge;

        let mut db = Database::new(Config::new().path("db/test/merge").temporary(true)).unwrap();

        db.bucket_create(prop);
        db.register_merge(prop, INT_MERGE_FN).unwrap();

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
