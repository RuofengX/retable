pub mod atom;
pub mod basic;
pub mod db;

use std::sync::Arc;

pub use basic::{Delta, Value, EID};
pub use db::Database;

/// merge方法的traits
pub type MergeFn = dyn Send + Sync + Fn(&mut Value, &Delta) -> ();
pub type MergeFnClosure = Arc<MergeFn>;

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

    /// 溢出错误
    #[error("Error when fmt str into marker.")]
    OverflowError,
}
