//! Basic types wrapper used in database.

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

/// The unique identifier for an entity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EID([u8; 8]);
impl EID {
    #[inline]
    /// Create a new EID from u64.
    ///
    /// Note that EID(0) is considered as invalid,
    /// which used to pre-heat the cache or insert entry, etc.
    pub fn new(value: u64) -> Self {
        EID::from(value)
    }
    #[inline]
    /// Create an empty EID, which is invalid.
    pub fn empty() -> Self {
        EID([0; 8])
    }

    #[inline]
    /// EID 0 is invalid.
    pub fn is_valid(&self) -> bool {
        const ZERO_ARR: [u8; 8] = [0; 8];
        ZERO_ARR != self.0
    }
}
impl From<u64> for EID {
    #[inline]
    fn from(value: u64) -> Self {
        EID(value.to_le_bytes())
    }
}

impl Into<u64> for EID {
    #[inline]
    fn into(self) -> u64 {
        u64::from_le_bytes(self.0)
    }
}
impl From<[u8; 8]> for EID {
    #[inline]
    fn from(value: [u8; 8]) -> Self {
        EID(value)
    }
}
impl AsRef<[u8]> for EID {
    #[inline]
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
    // DO NOT CHANGE THE ORDER.
    Bool(bool),       // 0
    EID(EID),         // 1
    UInt(u64),        // 2
    Int(i64),         // 3
    Float(f64),       // 4
    UInt3([u64; 3]),  // 5
    Int3([i64; 3]),   // 6
    Float3([f64; 3]), // 7
    UInt2([u64; 2]),  // 8
    Int2([i64; 2]),   // 9 
    Float2([f64; 2]), // 10
    Mark(Marker),     // 11
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
        Marker::from(hint)
    }
}
impl From<&str> for Marker {
    /// Create a marked value from info string.
    /// Return an Error when the length of str is greater than 31
    fn from(value: &str) -> Self {
        let buf = value.as_bytes();
        let mut len = buf.len();
        if len > 31 {
            len = 31
        };
        let mut v = [0u8; 31];
        v[..len].copy_from_slice(&buf[..len]);
        Marker(v)
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
        PropTag::from(hint)
    }
}

impl From<&str> for PropTag {
    /// Create a prop tag from string.
    /// Drop any character beyond 8.
    fn from(value: &str) -> Self {
        let mut len = value.len();
        if len > 8 {
            len = 8;
        }
        let mut v = [0u8; 8];
        v[..len].copy_from_slice(&value.as_bytes()[..len]);
        PropTag(v)
    }
}

impl AsRef<str> for PropTag {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}
