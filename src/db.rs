use std::io::{Write, Read};

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use ron::{self, ser::PrettyConfig};

use crate::{
    atom::{PropName, PropValue, EID, EntityProp},
    Prop, PropStorage,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Props{
    data: FxHashMap<PropName, Prop>,
    len: RwLock<usize>,
}

impl Props {
    pub fn new() -> Self {
        Props { data: FxHashMap::default(), len:RwLock::new(0) }
    }
    /// 从io中加载
    pub fn load(reader: impl Read) -> Result<Self, ron::error::SpannedError>{
        let data = ron::de::from_reader(reader)?;
        Ok(data)
    }
}
impl Props {
    /// 获取当前最大EID
    pub fn max_eid(&self) -> EID{
        let rtx = self.len.read();
        EID(*rtx)
    }
    /// 获取下一个EID
    pub fn next_eid(&self) -> EID{
        let rtx = self.len.read();
        EID(*rtx + 1)
    }
    /// 获取某一属性的底层存储
    pub fn get_prop(&self, key: &PropName) -> Option<&Prop>{
        self.data.get(&key)
    }

    /// 获取某一属性的底层存储的可变借用
    pub fn get_prop_mut(&mut self, key: &PropName) -> Option<&mut Prop>{
        self.data.get_mut(&key)
    }

    /// 列举出所有属性
    pub fn list_props(&self)-> &FxHashMap<PropName, Prop>{
        &self.data
    }

    /// 添加属性
    pub fn add_prop(&mut self, name: PropName, prop: Prop) -> Option<()>{
        if self.data.insert(name, prop).is_none(){
            Some(())
        } else {
            None
        }
    }

    /// 库中存在该入口 - 从给定的prop里获取一个实体属性值 - 不存在则None
    pub fn get<'a>(
        &'a self,
        eid: EID,
        key: &PropName,
    ) -> Option<&'a RwLock<PropValue>> {
        let rtx = self.len.read();
        if eid.0 >= *rtx{
            // 超过最大限制
            return None
        }
        // 没超过最大限制，获取不到就是None
        if let Some(prop) = self.data.get(key){
            prop.get(eid)
        } else {
            None
        }
    }

    /// 创建一个实体
    pub fn spawn(&mut self, props: EntityProp) -> EID {
        let mut wtx = self.len.write();
        let new_eid = EID(*wtx);
        *wtx += 1;

        for (key, value) in props{
            self.data
                .entry(key)
                .or_insert(Prop::default())
                .append(new_eid, value);
        }
        new_eid
    }


    /// 库中存在该入口 - 更新一个实体属性值 - 不存在则None
    /// 使用的是修改读写锁内的数据，而不是创建新的入口，复用了锁的功能，实现了线程安全
    pub fn update(&mut self, eid: EID, key: PropName, value: PropValue) -> Option<()> {
        {
            let rtx = self.len.write();
            if eid.0 >= *rtx{
                return None
            }
        }

        // 判断是否存在属性
        if let Some(inner_prop) = self.data.get(&key) {
            // 属性存在
            // 判断是否存在实体
            let prop_entry = inner_prop.get(eid).unwrap();
            // 实体在该属性上存在
            let mut wtx = prop_entry.write();
            *wtx = value;
            Some(())
        } else {
            // 实体在该属性上不存在
            None
        }
    }

    /// 删除eid实体的key属性
    pub fn remove(&mut self, eid:EID, key: PropName) -> Option<()>{
        let wtx = self.len.write();
        if eid.0 >= *wtx{
            return None
        }
        // 判断是否存在属性
        if let Some(inner_prop) = self.data.get_mut(&key) {
            //属性存在
            inner_prop.remove(eid)
        } else {
            // 实体在该属性上不存在
            None
        }


    }

    /// 保存至io中
    pub fn save(&self, writer: impl Write) -> Result<(), ron::Error>{
        let _rtx = self.len.read();
        ron::ser::to_writer_pretty(writer, &self, PrettyConfig::default())
    }

}

#[cfg(test)]
mod tests {
    use crate::scaler::IntVector3;

    use super::*;
    
    #[test]
    fn test_prop_value_sp() {
        let mut props = Props::default();

        let mut ent_0_prop: FxHashMap<PropName, PropValue> = FxHashMap::default();
        ent_0_prop.insert(
            PropName::Grid,
            PropValue::IntVec(IntVector3([0,0,0]))
        );

        // Test spawn
        assert_eq!(EID(0), props.spawn(ent_0_prop.clone()));
        assert_eq!(EID(1), props.spawn(FxHashMap::default()));
        assert_eq!(EID(2), props.spawn(ent_0_prop));
        let rtx = props.get(EID(0), &PropName::Grid).unwrap().read();
        let ent_0_grid= rtx.clone();
        assert_eq!(
            ent_0_grid,
            PropValue::IntVec(IntVector3([0;3]))
        );

        // Test save
        use std::fs::OpenOptions;
        let writer = OpenOptions::new().write(true).create(true).open(
            "./save.ron"
        ).unwrap();

        props.save(writer).unwrap();

        // Test load 
        let reader = OpenOptions::new().read(true).open(
            "./save.ron"
        ).unwrap();
        // assert_eq!(Props::load(reader).unwrap(), props); // 锁无法比较
        dbg!(Props::load(reader).unwrap());

    }
}

