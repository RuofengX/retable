use std::{io, fmt, error};


#[derive(Debug)]
pub enum Error{
    IOError(&'static str),
    SerializeError(&'static str),
    DeserializeError(&'static str),
    DuplicateKey(&'static str),
    KeyError(&'static str),
    ShardNotUseable(&'static str),
}
impl From<io::Error> for Error{
    fn from(_value: io::Error) -> Self {
        Self::IOError(&"输入/输出错误")
    }
}
impl fmt::Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl error::Error for Error{}