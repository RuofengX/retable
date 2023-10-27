use std::{sync::RwLock, collections::HashMap};

use crate::err::Error;
use shared_slab::Slab;

use crate::atom::{PropValue, EID};

struct PropPool{
    pool: Slab<RwLock<PropValue>>,
    index: RwLock<HashMap<EID, usize>>,
}
impl PropPool{
    /// 获取某一实体的属性
    pub fn get(&self, eid: EID) -> Option<&RwLock<PropValue>>{
        let index = self.index.read().unwrap();
        if let Some(uid) = index.get(&eid){
            let rtx = self.pool;
            return self.pool.get(*uid)
        } else {
            None
        }
    }

    /// 插入某一实体的属性
    pub fn insert(&self, eid: EID, value: PropValue) -> Result<(), Error>{
        let mut index = self.index.write().unwrap();
        if index.contains_key(&eid){
            return Err(Error::KeyError("尝试插入一个已存在的实体索引"))
        } else{
            if let Some(_) = index.insert(eid, self.pool.insert(RwLock::new(value))){
                Ok(())
            }
            else{
                Err(Error::ShardNotUseable("全局分片及当前线程分片已满，无法分配新属性槽"))
            }
        }
    }

    /// 删除某一属性, lazy_remove
    pub fn remove(&self, eid: EID) -> Result<(), Error>{
        let value: Option<usize>;
        {
            let mut wtx = self.index.read().unwrap();
            let id = wtx.remove(&eid);
        }
            if let Some(id) = value{
                self.pool.remove(id).unwrap();
                Ok(())
            } else {
                Err(Error::KeyError("尝试删除了一个不存在的实体编号"))
            }
    } 
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use shared_slab::Slab;

    #[bench]
    fn bench_insert(b: &mut Bencher) {
        let prop_pool = PropPool {
            pool: Slab::new(),
            index: RwLock::new(HashMap::new()),
        };

        b.iter(|| {
            let eid = 1;
            let value = PropValue::new();
            prop_pool.insert(eid, value).unwrap();
        });
    }

    #[bench]
    fn bench_get(b: &mut Bencher) {
        let prop_pool = PropPool {
            pool: Slab::new(),
            index: RwLock::new(HashMap::new()),
        };

        let eid = 1;
        let value = PropValue::new();
        prop_pool.insert(eid, value).unwrap();

        b.iter(|| {
            let prop = prop_pool.get(eid);
            assert!(prop.is_some());
        });
    }

    #[bench]
    fn bench_remove(b: &mut Bencher) {
        let prop_pool = PropPool {
            pool: Slab::new(),
            index: RwLock::new(HashMap::new()),
        };

        let eid = 1;
        let value = PropValue::new();
        prop_pool.insert(eid, value).unwrap();

        b.iter(|| {
            prop_pool.remove(eid).unwrap();
        });
    }
}
