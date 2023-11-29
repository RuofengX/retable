use crate::basic::{Value, EID, Marker};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct Index {
    pub eid: EID,
    pub prop: [u8; 8],
}
impl Index {
    pub fn new() -> Self {
        Index {
            eid: EID(0),
            prop: [0; 8],
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Atom {
    pub(crate) valid: bool,
    pub index: Index,
    pub value: Value,
}
impl Atom {
    pub fn empty() -> Self {
        Atom {
            index: Index {
                eid: EID(0),
                prop: [0; 8],
            },
            value: Value::Mark(Marker::new("NOT_SET")),
            valid: false,
        }
    }
    pub fn new(index: Index, value: Value) -> Self {
        Atom {
            index,
            value,
            valid: true,
        }
    }
}
