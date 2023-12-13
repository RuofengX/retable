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
        let v = write_str_into::<31>(value);
        Marker(v)
    }
}

impl AsRef<str> for Marker {
    fn as_ref(&self) -> &str {
        read_str_from(&self.0)
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
        let v = write_str_into::<8>(value);
        PropTag(v)
    }
}

impl AsRef<str> for PropTag {
    fn as_ref(&self) -> &str {
        read_str_from(&self.0)
    }
}

/// Write a string into a limited length buffer.
#[inline]
fn write_str_into<const N: usize>(raw: &str) -> [u8; N] {
    // init with 0, U+0000 equals to BLANK.
    let mut buf = [0u8; N];
    // the raw length.
    let len = raw.len();
    // the target length.
    let limit = N;

    if len == limit {
        // if len == limit -> no need to truncate, just copy.
        buf.copy_from_slice(&raw.as_bytes());
    } else if len < limit {
        // if len < limit -> no need to truncate, copy first len bytes.
        // rest bytes is set to 0.
        buf[..len].copy_from_slice(&raw.as_bytes()[..len]);
    } else {
        // if len > limit -> truncate, copy all bytes.
        let (len, trunc_str) = truncate_utf8(raw, limit);
        buf[..len].copy_from_slice(trunc_str);
    }
    buf
}

/// Convert utf8 string from a buffer
///
/// Drop all U+0000 and other bad thing.
/// Just read utf8 code before invalid occurs.
fn read_str_from<'t, T: AsRef<[u8]>>(buf: &'t T) -> &'t str {
    let buf = buf.as_ref();
    let rtn = match std::str::from_utf8(buf) {
        Ok(s) => s,
        Err(e) => unsafe { std::str::from_utf8_unchecked(&buf[..e.valid_up_to()]) },
    };
    rtn.trim_end_matches('\0')
}

/// max_length must <= s.len()
#[inline]
fn truncate_utf8(s: &str, max_length: usize) -> (usize, &[u8]) {
    let mut cut_index = 0;

    for (i, _) in s.char_indices().rev() {
        if i <= max_length {
            cut_index = i;
            break;
        }
    }

    (cut_index, &s.as_bytes()[..cut_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_utf8() {
        // Test case 1: String length is less than max_length
        let s1 = "Hello, World!";
        let max_length1 = 1;
        assert_eq!(truncate_utf8(s1, max_length1), (1, "H".as_bytes()));

        // Test case 2: String length is equal to max_length
        let s2 = "Hello, World!";
        let max_length2 = 3;
        assert_eq!(truncate_utf8(s2, max_length2), (3, "Hel".as_bytes()));

        // Test case 3: String length is greater than max_length
        let s3 = "Hello, World!";
        let max_length3 = 5;
        let expected_result3 = "Hello".as_bytes();
        assert_eq!(truncate_utf8(s3, max_length3), (5, expected_result3));

        // Test case 4: String contains multi-byte characters
        let s4 = "你好，世界！";
        let max_length4 = 6;
        let expected_result4 = "你好".as_bytes();
        assert_eq!(truncate_utf8(s4, max_length4), (6, expected_result4));
    }

    #[test]
    fn test_write_str_into() {
        // Test with a string that fits within the buffer
        let buffer = write_str_into::<5>("Hello");
        assert_eq!(&buffer, b"Hello");

        // Test with a string that needs truncation
        let buffer = write_str_into::<6>("世界你好");
        assert_eq!(&buffer, "世界".as_bytes());

        // Test with a string that needs truncation and includes multi-byte characters
        let buffer = write_str_into::<6>("こんにちは");
        assert_eq!(&buffer, "こん".as_bytes());

        // Test with a string that is longer than the buffer
        let buffer = write_str_into::<10>("This string is longer than the buffer length");
        assert_eq!(&buffer, "This strin".as_bytes());

        // Test with an empty string
        let buffer = write_str_into::<0>("");
        assert_eq!(&buffer[..0], "".as_bytes());
    }

    #[test]
    fn test_marker_new() {
        let marker = Marker::new("Hello, World!");
        assert_eq!(
            marker.0,
            "Hello, World!\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes()
        );

        let marker =
            Marker::new("你好世界你好世界你好世界你好世界你好世界你好世界你好世界你好世界你好世界");
        assert_eq!(marker.0, "你好世界你好世界你好\0".as_bytes());

        let marker = Marker::new("你好世界");
        assert_eq!(marker.0, "你好世界\0\0\0\0\0\0\0\0\0\0\0\0\0".as_bytes());
    }

    #[test]
    fn test_marker_eq() {
        let marker1 = Marker::new("Hello, World!");
        let marker2 = Marker::new(
            "Hello, World!\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        );
        let marker3 = Marker::new("Hello, World!\0\0\0");
        assert_eq!(marker1, marker2);
        assert_eq!(marker1, marker3);
        assert_eq!(marker2, marker3);

        let marker4 = Marker::new("Hello,\0 World!");
        let marker5 =
            Marker::new("Hello,\0 World!\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0你好");
        assert_eq!(marker4, marker5);
    }

    #[test]
    fn test_marker_from() {
        let marker = Marker::from("Hello, World!");
        assert_eq!(marker, Marker::new("Hello, World!"));

        let long_string =
            "This is a very long string that exceeds the maximum length of 31 characters.";
        let marker = Marker::from(long_string);
        assert_eq!(marker.as_ref(), "This is a very long string that");
    }

    #[test]
    fn test_marker_as_ref() {
        let marker = Marker::new("Hello, World!");
        assert_eq!(marker.as_ref(), "Hello, World!");

        let marker = Marker::new(
            "a你好世界你好世界你好世界你好世界你好世界你好世界你好世界你好世界你好世界",
        );
        assert_eq!(marker.as_ref(), "a你好世界你好世界你好"); // len = 31

        let marker = Marker::new(
            "aa你好世界你好世界你好世界你好世界你好世界你好世界你好世界你好世界你好世界",
        );
        assert_eq!(marker.as_ref(), "aa你好世界你好世界你"); // len = 29

        let marker = Marker::new("你好世界");
        assert_eq!(marker.as_ref(), "你好世界");
    }
}
