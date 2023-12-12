//! Basic types wrapper used in database.
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// The unique identifier for an entity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EID([u8; 8]);
impl EID {
    #[allow(missing_docs)]
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

/// A mark that could convert into [`Value`]
#[enum_dispatch]
pub trait Valuable {}

/// An enum data structure that can be stored in a bucket.
#[allow(missing_docs)]
#[enum_dispatch(Valuable)]
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

/// A delta is a change to a value.
///
/// User define the merge function to merge the delta with the current value.
pub type Delta = Value;

/// A limited length string that used as a marker.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Marker([u8; 31]);
impl Marker {
    /// Create a marked value from info string.
    ///
    /// cut off any character after 31.
    pub fn new(hint: &str) -> Self {
        let mut v = [0u8; 31];
        v.copy_from_slice(&hint.as_bytes()[..31]);
        Marker(v)
    }
}
impl TryFrom<&'static str> for Marker {
    type Error = Error;

    /// Create a marked value from info string.
    /// Return an Error when the length of str is greater than 30
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut v = [0u8; 31];
        let buf = value.as_bytes();
        if buf.len() > 31 {
            return Err(Error::OverflowError);
        }
        v.copy_from_slice(&buf[..31]);
        Ok(Marker(v))
    }
}
impl AsRef<str> for Marker {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

/// A limited length string that used as a name of prop.
///
/// Cheap workaround to make the name of prop Copy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PropTag([u8; 8]);
impl PropTag {
    /// Create a marked prop tag from info string.
    ///
    /// cut off any character after 8.
    pub fn new(hint: &str) -> Self {
        let mut v = [0u8; 8];
        v.copy_from_slice(&hint.as_bytes()[..8]);
        PropTag(v)
    }
}
impl TryFrom<&'static str> for PropTag {
    type Error = Error;

    /// Create a prop tag from string.
    /// Return an Error when the length of str is greater than 30
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut v = [0u8; 8];
        let buf = value.as_bytes();
        if buf.len() > 8 {
            return Err(Error::OverflowError);
        }
        v.copy_from_slice(&buf[..8]);
        Ok(PropTag(v))
    }
}
impl AsRef<str> for PropTag {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}
