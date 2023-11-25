pub mod db;
pub mod basic;
pub mod atom;

use std::sync::Arc;

use basic::{Value, EID, Delta};

/// prop存储方案必须要实现的特质
/// 对单一属性的存储方案的签名
pub trait AtomStorage{
    // 为eid新增一个属性，eid永远是新增的
    fn set(&mut self, eid: EID, prop:&str, value: Value) -> ();

    // 获取eid的属性
    fn get(&self, eid: EID, prop: &str) -> Option<Value>;

    // 删除eid的属性
    fn remove(&mut self, eid: EID, prop: &str) -> Option<()>;

    // 注册merge函数
    fn register_merge<F>(&mut self, prop: &str, f: Arc<F>)
    where
        F: FnMut(EID, &Delta) -> ();
    
    // 使用merge函数合并属性
    fn merge(&mut self, prop: &str, eid: EID, delta: &Delta) -> ();

    // TODO: 添加输入、输出流

}
