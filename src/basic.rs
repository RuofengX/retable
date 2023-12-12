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
        let mut v: [u8; 31] = [0; 31];
        write_str_into(value, &mut v);
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
        let mut v = [0u8; 8];
        write_str_into(value, &mut v);
        PropTag(v)
    }
}

impl AsRef<str> for PropTag {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

/// Write a string into a limited length buffer.
#[inline]
fn write_str_into<T: AsMut<[u8]>>(raw: &str, mut buf: T) {
    let len = buf.as_mut().len();
    let limit = buf.as_mut().len();

    if len <= limit {
        buf.as_mut().copy_from_slice(&raw.as_bytes()[..limit]);
    } else {
        let trunc_str = truncate_utf8(raw, limit);
        buf.as_mut().copy_from_slice(trunc_str);
    }
}

#[inline]
fn truncate_utf8(s: &str, max_length: usize) -> &[u8]{
    let mut char_count = 0;
    let mut byte_count = 0;

    for (i, _) in s.char_indices() {
        char_count += 1;
        byte_count = i + 1;

        if char_count > max_length || !s.is_char_boundary(i + 1) {
            break;
        }
    }

    &s.as_bytes()[..byte_count]
    todo!("test needed");
}
