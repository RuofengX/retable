use std::{sync::RwLock, collections::HashMap};

use crate::{err::Error, atom::PropValue};
use sharded_slab::{Slab, Entry};

use crate::atom::EID;

pub struct IndexSlab{
    pool: Slab<PropValue>,
    index: RwLock<HashMap<EID, usize>>,
}
impl IndexSlab{
    /// new
    pub fn new() -> Self{
        Self { pool: Slab::new(), index: RwLock::new(HashMap::new()) }
    }
    /// 获取某一实体的属性
    pub fn get(&self, eid: EID) -> Option<Entry<PropValue>>{
        let index = self.index.read().unwrap();
        if let Some(uid) = index.get(&eid){
            self.pool.get(*uid)
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
            if let Some(aid) = self.pool.insert(value){
                index.insert(eid, aid);
                Ok(())
            }
            else{
                Err(Error::ShardNotUseable("全局分片及当前线程分片已满，无法分配新属性槽"))
            }
        }
    }

    /// 删除某一属性, lazy_remove
    pub fn remove(&self, eid: EID) -> Result<(), Error>{
        let mut wtx = self.index.write().unwrap();
        let value = wtx.remove(&eid);
        if let Some(id) = value{
            if self.pool.remove(id){
                Ok(())
            } else {
                Err(Error::KeyError("尝试删除了一个已被（其他线程）删除实体编号"))
            }
        } else {
            Err(Error::KeyError("尝试删除了一个不存在的实体编号"))
        }
    } 
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use super::*;
    use crate::atom::PropValue;
    
    #[test]
    fn test_index_slad() {

        let slad: IndexSlab = IndexSlab {
            pool: Slab::new(),
            index: RwLock::new(HashMap::new()),
        };
        
        // Test insert and get
        slad.insert(EID(1), PropValue::Int(1024)).unwrap();
        assert_eq!(*slad.get(EID(1)).unwrap(), PropValue::Int(1024));
        
        // Test insert duplicate key error
        let result = slad.insert(EID(1), PropValue::Int(2048));
        assert!(result.is_err());
        
        // Test remove
        slad.remove(EID(1)).unwrap();
        assert!(slad.get(EID(1)).is_none());
        
        // Test remove non-existent key error
        let result = slad.remove(EID(1));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_index_slad_concurrent() {
        use std::thread;
        
        const THREADS: u64 = 64;
        
        let slab = Arc::new(IndexSlab {
            pool: Slab::new(),
            index: RwLock::new(HashMap::new()),
        });
        
        let handles: Vec<_> = (0..THREADS).map(|i| -> thread::JoinHandle<()> {
            // println!("{}", i);
            let slab_clone = Arc::clone(&slab);
            thread::spawn(move ||{
                let _ = slab_clone.insert(EID(i), PropValue::UInt(i));
                assert_eq!(*slab_clone.get(EID(i)).unwrap(), PropValue::UInt(i));
                thread::sleep(Duration::new(1,0));
                if i > 0{
                    assert_eq!(*slab_clone.get(EID(i-1)).unwrap(), PropValue::UInt(i-1));
                }
                thread::sleep(Duration::new(1,0));
                slab_clone.remove(EID(i)).unwrap();
                thread::sleep(Duration::new(1,0));
                if i > 1{
                    assert!(slab_clone.get(EID(i-1)).is_none());
                }
                assert!(slab_clone.get(EID(i)).is_none());
            })}
        ).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
