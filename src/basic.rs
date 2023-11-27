use as_bytes::AsBytes;
use kv::Key;
use rustc_hash::FxHashMap;
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Bool(bool),
    EID(EID),
    UInt(u64),
    Int(i64),
    Float(f64),
    String(String),
    UInt3([u64; 3]),
    Int3([i64; 3]),
    Float3([f64; 3]),
    UInt2([u64; 2]),
    Int2([i64; 2]),
    Float2([f64; 2]),
    List(Vec<Value>),
    Map(FxHashMap<EID, Value>),
}

/// A delta is a change to a value.
/// User define the merge function to merge the delta with the current value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Delta {
    Bool(bool),
    EID(EID),
    UInt(u64),
    Int(i64),
    Float(f64),
    String(String),
    UInt3([u64; 3]),
    Int3([i64; 3]),
    Float3([f64; 3]),
    UInt2([u64; 2]),
    Int2([i64; 2]),
    Float2([f64; 2]),
    List(Vec<Value>),
    Map(FxHashMap<EID, Value>),
}