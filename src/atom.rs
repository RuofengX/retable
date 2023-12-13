//! ATOM, Atomic Type Object Model
#![allow(missing_docs)]
use crate::basic::{Marker, PropTag, Value, EID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct Index {
    pub eid: EID,
    pub prop: [u8; 8],
}
impl Index {
    pub fn new() -> Self {
        Index {
            eid: EID::new(0),
            prop: [0; 8],
        }
    }
}

/// Atom, abstruct type object model.
///
/// Bincode version 2 is needed for correct enum serialize,
/// see more in [bincode-org/bincode](https://github.com/bincode-org/bincode/tree/trunk#why-does-bincode-not-respect-repru8)
///
///# Example
/// ```rust
///
/// use std::fs::File;
/// use std::io::Write;
/// use retable::atom::Atom;
/// use retable::basic::{EID, PropTag, Value, Marker};
///
/// let a = Atom{eid:EID::new(1), prop:PropTag::new("god"), value:Value::Mark(Marker::new("HELLO_WORLD这是UTF88芭芭芭芭芭"))};
/// let buf = bincode::serde::encode_to_vec::<Atom, bincode::config::Configuration>(a, bincode::config::Configuration::default()).unwrap();
///
/// // 创建一个文件并打开以写入模式
/// let mut file = File::create("output.atom").expect("无法创建文件");
/// // 将字符串写入文件
/// file.write_all(&buf).expect("写入文件时出错");
///
/// ```
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Atom {
    pub eid: EID,
    pub prop: PropTag,
    pub value: Value,
}

impl Atom {
    pub fn empty() -> Self {
        Atom {
            eid: EID::new(0),
            prop: PropTag::new("EMPTY"),
            value: Value::Mark(Marker::new("NOT_SET")),
        }
    }
    pub fn new(eid: EID, prop: PropTag, value: Value) -> Self {
        Atom { eid, prop, value }
    }
    pub fn valid(&self) -> bool {
        self.eid.is_valid()
    }
}
