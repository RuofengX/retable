use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::{
    atom::{PropName, PropValue, EID, EntityProp},
    PropStorage,
};

pub struct Props<T>
where T: PropStorage{
    data: FxHashMap<PropName, T>,
    len: RwLock<usize>,
}

impl <T: PropStorage>Props<T> {
    pub fn new() -> Self {
        Props { data: FxHashMap::default(), len:RwLock::new(0) }
    }
}
impl<T: PropStorage> Props<T> {
    /// 获取某一属性的底层存储
    pub fn get_prop(&self, key: &PropName) -> Option<&T>{
        self.data.get(&key)
    }

    /// 获取某一属性的底层存储的可变借用
    pub fn get_prop_mut(&mut self, key: &PropName) -> Option<&mut T>{
        self.data.get_mut(&key)
    }

    /// 列举出所有属性
    pub fn list_props(&self)-> &FxHashMap<PropName, T>{
        &self.data
    }

    /// 添加属性
    pub fn add_prop(&mut self, name: PropName, prop: T) -> Option<()>{
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
                .or_insert(T::default())
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

}
