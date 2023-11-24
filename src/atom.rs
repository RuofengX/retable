use serde::{Deserialize, Serialize};
use crate::basic::{Value, EID};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Index{
    pub eid: EID,
    pub prop: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom{
    pub index: Index,
    pub value: Value,
}
