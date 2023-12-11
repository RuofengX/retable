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
