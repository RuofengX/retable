pub mod db;
pub mod basic;
pub mod atom;

use std::sync::Arc;

use basic::{Value, EID, Delta};

/// merge方法的traits
pub trait MergeFn: Fn(&mut Value, &Delta) -> (){}
type MergeFnClosure = Arc<dyn MergeFn>;

/// prop存储方案必须要实现的特质
/// 对单一属性的存储方案的签名
pub trait AtomStorage{
    /// 获取eid的属性
    /// kv实现内部可变性
    fn get(&self, eid: EID, prop: &str) -> Option<Value>;

    /// 为eid的prop属性设置一个数值
    /// kv实现内部可变性
    fn set(&self, eid: EID, prop:&str, value: Value, retrieve: bool) -> Option<Value>;

    /// 删除eid的属性
    /// kv实现内部可变性
    fn remove(&self, eid: EID, prop: &str, retrieve: bool) -> Option<Value>;

    /// 注册merge函数
    fn register_merge(&mut self, prop: &str, f: Arc<dyn MergeFn>);
    
    /// 使用merge函数合并属性，
    /// 为最大化性能抛弃所有结果
    fn merge(&self, prop: &str, eid: EID, delta: Delta) -> ();

    // TODO: 添加输入、输出流

}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// 根据所给key未找到入口 
    #[error("Key [{0}] not exists.")]
    KeyError(String),

    /// 底层数据库错误
    #[error("Error from kv database.")]
    KvError(#[from] kv::Error),

    /// 底层数据库错误
    #[error("Error from sled database.")]
    SledError(#[from] sled::Error),
}