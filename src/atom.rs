use rand::Rng;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::scaler::Vec3;

/// 实体的ID
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash,
)]
pub struct EID(pub usize);
impl EID {
    pub fn new(eid: usize) -> Self {
        Self(eid)
    }
    pub fn rand(mut rng: impl Rng) -> Self {
        Self(rng.gen())
    }
    pub fn range(i: usize) -> impl Iterator<Item = EID> {
        (0..i).map(|i| Self(i))
    }
    pub fn next(&self) -> EID {
        EID(self.0 + 1)
    }
}

/// Atom记录的ID
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash,
)]
pub struct AID(pub usize);
impl AID {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(
    Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy,
)]
pub enum PropName {
    Grid,
    Pos,
    Infomation,
    DataLoss,
    #[default]
    None,
    // TODO在这里添加新的属性枚举
}
/// 从字符串生成属性名
impl From<&str> for PropName {
    fn from(value: &str) -> Self {
        match value {
            "pos" => Self::Pos,
            "grid" => Self::Grid,
            &_ => Self::DataLoss,
        }
    }
}

/// 属性值
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Clone)]
pub enum PropValue {
    F64V(Vec3::<f64>),
    I64V(Vec3::<i64>),
    I64(i64),
    U64(u64),
    EID(EID),
    EIDV(Vec<EID>),
    Str(String),
    List(Vec<Box<PropValue>>),
    #[default]
    Zero,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Atom {
    // 实体ID
    pub eid: EID,
    pub raw_atom: RawAtom,
}
impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.eid == other.eid
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RawAtom {
    // 属性
    pub prop_name: PropName,
    // 属性值
    pub prop_value: PropValue,
}

pub type EntityProp = FxHashMap<PropName, PropValue>;
