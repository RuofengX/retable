use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::atom::PropValue;
use sharded_slab::{Entry, Slab};

use crate::atom::EID;

pub struct PropValueSlab {
    pool: Slab<RwLock<PropValue>>,
    index: RwLock<FxHashMap<EID, usize>>,
}
impl PropValueSlab {
    /// new
    pub fn new() -> Self {
        Self {
            pool: Slab::new(),
            index: RwLock::new(FxHashMap::default()),
        }
    }

    /// 获取某一实体的属性，直接返回RwLock包装的值
    pub fn get(&self, eid: EID) -> Option<Entry<RwLock<PropValue>>> {
        let index = self.index.read();
        if let Some(uid) = index.get(&eid) {
            self.pool.get(*uid)
        } else {
            None
        }
    }

    /// 插入某一实体的属性
    pub fn insert(&self, eid: EID, value: PropValue) -> Option<()> {
        let mut index = self.index.write();
        if index.contains_key(&eid) {
            // 已有值则不做修改
            return None;
        } else {
            if let Some(aid) = self.pool.insert(RwLock::new(value)) {
                index.insert(eid, aid);
                Some(())
            } else {
                // Err(Error::ShardNotUseable("全局分片及当前线程分片已满，无法分配新属性槽")) //不是预料中的
                panic!("全局分片及当前线程分片已满，无法分配新属性槽")
            }
        }
    }

    /// 删除某一属性, lazy_remove
    pub fn remove(&self, eid: EID) -> Option<()> {
        let mut wtx = self.index.write();
        let value = wtx.remove(&eid);
        if let Some(id) = value {
            if self.pool.remove(id) {
                Some(())
            } else {
                // Err(Error::KeyError("尝试删除了一个已被（其他线程）删除实体编号"))
                None
            }
        } else {
            // Err(Error::KeyError("尝试删除了一个不存在的实体编号"))
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use super::*;
    use crate::atom::PropValue;

    #[test]
    fn test_index_slab() {
        let slab: PropValueSlab = PropValueSlab {
            pool: Slab::new(),
            index: RwLock::new(FxHashMap::default()),
        };

        // Test insert and get
        slab.insert(EID(1), PropValue::Int(1024)).unwrap();
        assert_eq!(
            *slab.get(EID(1)).unwrap().read(),
            PropValue::Int(1024)
        );

        // Test insert duplicate key error
        let result = slab.insert(EID(1), PropValue::Int(2048));
        assert!(result.is_none());

        // Test remove
        slab.remove(EID(1)).unwrap();
        assert!(slab.get(EID(1)).is_none());

        // Test remove non-existent key error
        let result = slab.remove(EID(1));
        assert!(result.is_none());
    }

    #[test]
    fn test_index_slab_concurrent() {
        use std::thread;

        const THREADS: u64 = 64;

        let slab = Arc::new(PropValueSlab {
            pool: Slab::new(),
            index: RwLock::new(FxHashMap::default()),
        });

        let handles: Vec<_> = (0..THREADS)
            .map(|i| -> thread::JoinHandle<()> {
                // println!("{}", i);
                let slab_clone = Arc::clone(&slab);
                thread::spawn(move || {
                    let _ = slab_clone.insert(EID(i), PropValue::UInt(i));
                    assert_eq!(
                        *slab_clone.get(EID(i)).unwrap().read(),
                        PropValue::UInt(i)
                    );
                    thread::sleep(Duration::new(1, 0));
                    if i > 0 {
                        assert_eq!(
                            *slab_clone.get(EID(i - 1)).unwrap().read(),
                            PropValue::UInt(i - 1)
                        );
                    }
                    thread::sleep(Duration::new(1, 0));
                    slab_clone.remove(EID(i)).unwrap();
                    thread::sleep(Duration::new(1, 0));
                    if i > 1 {
                        assert!(slab_clone.get(EID(i - 1)).is_none());
                    }
                    assert!(slab_clone.get(EID(i)).is_none());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
