use std::sync::RwLock;

use sharded_slab::Entry;

use crate::atom::{PropValue, EID, PropName};

pub trait Turkey{
    /// 无条件 - 覆盖设置一个实体属性值，等价于insert + update - 不存在则None, TODO:分片满时可能panic
    fn set(&mut self, eid:EID, key:PropName, value: PropValue) -> Option<()>;

    /// 库中不存在该入口 - 插入一个实体属性值 - 不存在则None, 分片满时可能panic
    fn insert(&mut self, eid:EID, key:PropName, value: PropValue) -> Option<()>;

    /// 库中存在该入口 - 获取一个实体属性值 - 不存在则None
    fn get(&self, eid:EID, key:PropName) -> Option<Entry<RwLock<PropValue>>>;

    /// 库中存在该入口 - 更新一个实体属性值 - 不存在则None
    fn update(&mut self, eid:EID, key:PropName, value: PropValue) -> Option<()>;

    /// 库中存在该入口 - 删除一个实体属性值，如不存在则返回None
    fn drop(&mut self, eid:EID, key:PropName) -> Option<()>;


}