pub mod atom;
pub mod basic;
pub mod db;

pub use basic::{Delta, Value, EID};
pub use db::Database;
use typed_sled::Tree;

/// merge方法的traits
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

/// 存储类型
// TODO: 日后实现Dense存储会调整这个类型
pub type PropBucket = Tree<EID, Value>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// 根据所给prop未找到属性库
    #[error("Prop [{0}] not exists.")]
    PropError(String),

    /// 根据所给key未找到入口
    #[error("Key [{0}] not exists.")]
    KeyError(String),

    /// 底层数据库错误
    #[error("Error from sled database.")]
    SledError(#[from] sled::Error),

    /// 溢出错误
    #[error("Error when fmt str into marker.")]
    OverflowError,
}
