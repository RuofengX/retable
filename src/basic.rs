use serde::{Deserialize, Serialize};

use crate::Error;

/// An EID is a unique identifier for an entity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EID([u8; 8]);
impl EID {
    pub fn new(value: u64) -> Self {
        EID::from(value)
    }
}
impl From<u64> for EID {
    fn from(value: u64) -> Self {
        let raw_bytes = value.to_le_bytes();
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&raw_bytes);
        EID(buf)
    }
}

impl Into<u64> for EID {
    fn into(self) -> u64 {
        u64::from_le_bytes(self.0)
    }
}
impl From<[u8; 8]> for EID {
    fn from(value: [u8; 8]) -> Self {
        EID(value)
    }
}
impl AsRef<[u8]> for EID {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A value is a data structure that can be stored in a bucket.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    EID(EID),
    UInt(u64),
    Int(i64),
    Float(f64),
    UInt3([u64; 3]),
    Int3([i64; 3]),
    Float3([f64; 3]),
    UInt2([u64; 2]),
    Int2([i64; 2]),
    Float2([f64; 2]),
    Mark(Marker), // The maxium size of any variable.
}

/// A delta is a change to a value.  /// User define the merge function to merge the delta with the current value.
pub type Delta = Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Marker([u8; 30]);
impl Marker {
    /// Create a marked value from info string.
    /// cut off any character after 30.
    pub fn new(hint: &str) -> Self {
        let mut v = [0u8; 30];
        v.copy_from_slice(&hint.as_bytes()[..30]);
        Marker(v)
    }
}
impl TryFrom<&'static str> for Marker {
    type Error = Error;

    /// Create a marked value from info string.
    /// Return an Error when the length of str is greater than 30
    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        let mut v = [0u8; 30];
        let buf = value.as_bytes();
        if buf.len() > 30 {
            return Err(Error::OverflowError);
        }
        v.copy_from_slice(&buf[..30]);
        Ok(Marker(v))
    }
}

pub trait PropStorage {
    /// 获取存储的名称
    fn name(&self) -> &'static str;

    /// 获取eid的属性
    fn get(&self, eid: EID) -> Option<Value>;

    /// 为eid的prop属性设置一个数值
    fn set(&self, eid: EID, value: Value, retrieve: bool) -> Option<Value>;

    /// 为eid的prop属性设置一个数值，
    /// 如不存在则生成新的
    fn set_or_insert(&mut self, eid: EID, value: Value, retrieve: bool) -> Option<Value>;

    /// 删除eid的属性，
    /// kv实现内部可变性
    fn remove(&self, eid: EID, retrieve: bool) -> Option<Value>;

    /// 注册merge函数，如果`prop`不存在，则将返回一个`Error::PropError`
    fn register_merge(&mut self, f: MergeFn) -> Result<(), Error>;

    /// 使用merge函数合并属性，
    /// 为最大化性能抛弃所有结果
    fn merge(&self, eid: EID, delta: Delta) -> ();

    /// 注册一个tick函数，如果`prop`不存在，则将返回一个`Error::PropError`
    fn register_tick(&mut self, f: TickFn) -> Result<(), Error>;

    /// 调用一个prop上的tick方法
    fn tick(&self);

    // TODO: 添加批量merge操作
    // TODO: 添加输入、输出流
    // TODO: 添加默认的merge函数
}

/// prop存储方案必须要实现的特质
/// 对单一属性的存储方案的签名
pub trait AtomStorage: Sync + Send {
    fn get_prop<'s>(&'s self, prop: &'static str) -> Option<&'s dyn PropStorage>;

    fn get_prop_or_create<'s>(&'s mut self, prop: &'static str) -> &'s dyn PropStorage;
}

/// merge方法的类型
/// merge方法是由外部更新
///
/// Example
/// ```
/// Fn(
///     EID,           // merge发生的EID
///     Option::<Value>, // 当前Value，如果有
///     Delta,         // 传入的Delta
/// ) -> Option::<Value> // 返回新的Value，返回None表示删除原有Value
/// ```
pub type MergeFn = fn(EID, Option<Value>, Delta) -> Option<Value>;

/// tick方法
/// 输入当前EID，当前Value，以及属性库
/// 返回新的Delta
pub type TickFn = fn(EID, Value, &PropBucket) -> Option<Delta>;

