use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::{
    atom::{PropName, PropValue, EID},
    prop_slab::PropValueSlab,
};

pub struct Props(FxHashMap<PropName, PropValueSlab>);
impl Props {
    pub fn new() -> Self {
        Props { 0: FxHashMap::default() }
    }
}
impl Props {
    /// 无条件 - 覆盖设置一个实体属性值，等价于insert + update - 不存在则None
    /// 性能弱
    //  TODO:分片满时可能panic
    pub fn set(
        &mut self,
        eid: crate::atom::EID,
        key: PropName,
        value: crate::atom::PropValue,
    ) -> Option<()> {
        // 获取该属性的IndexSlab入口
        let prop_slab = self.0.entry(key).or_insert(PropValueSlab::new());

        // 尝试插入，如不存在则更新
        if let Some(prop_entry) = prop_slab.get(eid) {
            // 获取到了实体的属性入口
            *prop_entry.write() = value;
            Some(())
        } else {
            // 实体在该属性上不存在
            prop_slab.insert(eid, value) // 直接返回
        }
    }

    /// 库中不存在该入口 - 插入一个实体属性值 - 不存在则None
    /// 性能弱
    //  TODO:分片满时可能panic
    pub fn insert(&mut self, eid: EID, key: PropName, value: PropValue) -> Option<()> {
        self.0
            .entry(key)
            .or_insert(PropValueSlab::new())
            .insert(eid, value)
    }

    /// 库中存在该入口 - 获取一个实体属性值 - 不存在则None
    pub fn get(
        &self,
        eid: EID,
        key: PropName,
    ) -> std::option::Option<sharded_slab::Entry<RwLock<PropValue>>> {
        if let Some((_, prop_slab)) = self.0.get_key_value(&key) {
            prop_slab.get(eid)
        } else {
            // 不存在该属性，也不会改变原字典
            None
        }
    }

    /// 库中存在该入口 - 更新一个实体属性值 - 不存在则None
    pub fn update(&mut self, eid: EID, key: PropName, value: PropValue) -> Option<()> {
        // 判断是否存在属性
        if let Some(prop_slab) = self.0.get(&key) {
            // 属性存在

            // 判断是否存在实体
            if let Some(prop_entry) = prop_slab.get(eid) {
                // 实体在该属性上存在
                *prop_entry.write() = value;
                Some(())
            } else {
                // 实体在该属性上不存在
                None
            }
        } else {
            // 该属性不存在
            None
        }
    }

    /// 库中存在该入口 - 删除一个实体属性值，如不存在则返回None
    pub fn drop(&mut self, eid: EID, key: PropName) -> Option<()> {
        // 判断是否存在属性
        if let Some(prop_slab) = self.0.get(&key) {
            // 属性存在，删除id对应记录
            prop_slab.remove(eid)
        } else {
            // 该属性不存在
            None
        }
    }
}
