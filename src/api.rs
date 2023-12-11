/// A set of trait that defines the behavior of database and property storage.
use std::sync::Arc;

use crate::{
    basic::{Delta, Value, EID},
    error::Error,
    method::{MergeFn, TickFn},
};

/// A trait that defines the behavior of property storage.
///
/// 'prop' means that the storage is only for one property.
pub trait PropStorage {
    /// 获取存储的名称
    fn name(&self) -> &str;

    /// 获取eid的属性
    fn get(&self, eid: &EID) -> Option<Value>;

    /// 为eid的prop属性设置一个数值
    fn set(&self, eid: &EID, value: Value, retrieve: bool) -> Option<Value>;

    /// 删除eid的属性，
    /// kv实现内部可变性
    fn remove(&self, eid: &EID, retrieve: bool) -> Option<Value>;

    /// 注册merge函数，如果`prop`不存在，则将返回一个`Error::PropError`
    fn register_merge(&mut self, f: MergeFn) -> Result<(), Error>;

    /// 使用merge函数合并属性，
    /// 为最大化性能抛弃所有结果
    fn merge(&self, eid: &EID, delta: Delta) -> ();

    /// 注册一个tick函数，如果`prop`不存在，则将返回一个`Error::PropError`
    fn register_tick(&mut self, f: TickFn) -> Result<(), Error>;

    /// 调用一个prop上的tick方法
    fn tick(&self);

    // TODO: 添加批量merge操作
    // TODO: 添加输入、输出流
    // TODO: 添加默认的merge函数
}

/// The trait that design for database storage.
pub trait AtomStorage {
    fn get_prop(&self, prop: &'static str) -> Option<Arc<dyn PropStorage>>;

    fn create_prop(
        &mut self,
        prop: &'static str,
        merge: MergeFn,
        tick: TickFn,
    ) -> Arc<dyn PropStorage>;
}
