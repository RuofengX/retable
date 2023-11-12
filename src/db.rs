use std::{path::Path, io::Write, sync::atomic::{AtomicU64, Ordering}};

use sled::{self, IVec, Tree};

use crate::types::{storage::{PropValue, AtomData}, base::EID, atom::Atom};

pub struct Props{
    db: sled::Db,
    len: AtomicU64,
}
impl Props{
    pub fn new(path: &Path)->Option<Props>{
        if let Ok(db) = sled::open::<&Path>(path){
            Some(Props{
                db,
                len: AtomicU64::new(0),
            })
        } else {
            None
        }

    }
}
impl Props{
    /// 获取当前最大EID
    pub fn len(&self) -> u64 {
        self.len.load(Ordering::AcqRel)
    }
    fn len_next(&mut self) {
        self.len.fetch_add(1, Ordering::AcqRel);
    }

    /// 获取属性入口点
    pub fn get_prop(&mut self, key: &str) -> Option<Tree>{
        if self.db.tree_names().contains(&IVec::from(key)){
            Some(self.db.open_tree(key).unwrap())
        } else {
            None
        }
    }

    /// 添加属性
    pub fn add_prop(&mut self, key: &str) -> Option<()> {
        if self.db.tree_names().contains(&IVec::from(key)){
            None
        } else {
            self.db.open_tree(key);
            Some(())
        }

    }

    /// 库中存在该入口 - 从给定的schema里获取一个实体属性值 - 不存在则None
    /// 返回后的属性还需要再进行类型转换
    pub fn get(&self, eid: u64, schema: AtomData) -> Option<AtomData> {
        if eid >= self.len(){
            // 超过最大限制
            return None;
        }
        // 没超过最大限制，获取不到就是None
        if let Ok(tree) = self.db.open_tree::<&[u8]>(&[schema.value_type().0]){
            let eid = EID::new(eid);
            if let Ok(value) = tree.get::<&[u8;8]>(&eid.0){
                if let Some(value) = value{
                    let value = flatbuffers::root::<AtomData>(&value).unwrap();
                    return Some(value)
                } 
            }
        }
        None
    }

    /// 创建一个实体
    pub fn spawn(&mut self, props: Vec<AtomData>) -> u64{
        self.len_next();
        todo!()
    }

    /// 库中存在该入口 - 更新一个实体属性值 - 不存在则None
    /// 使用的是修改读写锁内的数据，而不是创建新的入口，复用了锁的功能，实现了线程安全
    pub fn update(&mut self, eid: EID, key: &str, value: PropValue) -> Option<()> {
        todo!()
    }

    /// 删除eid实体的key属性
    pub fn remove(&mut self, eid: EID, key: &str) -> Option<()> {
        todo!()
    }

    /// 保存至io中
    pub fn save(&self, writer: impl Write) -> Result<(), ron::Error> {
        todo!()
    }
}