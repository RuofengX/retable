use as_bytes::AsBytes;
use kv::Key;
use serde::{Deserialize, Serialize};

/// An EID is a unique identifier for an entity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EID(pub usize);
impl AsRef<[u8]> for EID {
    fn as_ref(&self) -> &[u8] {
        unsafe { self.as_bytes() }
    }
}
impl<'a> Key<'a> for EID {
    fn from_raw_key(r: &'a kv::Raw) -> Result<Self, kv::Error> {
        match r.len() {
            32 => Ok(EID(r as *const _ as usize)),
            _ => Err(kv::Error::Message("错误的EID二进制".into())),
        }
    }
}

/// A value is a data structure that can be stored in a bucket.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    Mark([u8; 30]), // The maxium size of any variable.
}

/// A delta is a change to a value.
/// User define the merge function to merge the delta with the current value.
pub type Delta = Value;
