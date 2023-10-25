use std::sync::RwLock;

use serde::{Serialize, Deserialize};

/// 实体的ID
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EID(u64);
impl EID{
    pub fn new(eid:u64) -> Self{
        Self(eid)
    }
}

/// Atom记录的ID
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct AID(usize);
impl AID{
    pub fn new(id:usize) -> Self{
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub enum PropName{
    Grid,
    Pos,
    DataLoss,
    #[default]
    None,
    // TODO在这里添加新的属性枚举
}
impl From<&str> for PropName{
    fn from(value: &str) -> Self {
        match value{
            "pos" => Self::Pos,
            "grid" => Self::Grid,
            &_ => Self::DataLoss,
        }
    }
}

/// 属性值
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd)]
pub enum PropValue{
    Str(String),
    Vec([f64;3]),
    IntVec([i64;3]),
    Int(i64),
    UInt(u64),
    Document(Box<PropValue>),
    #[default]
    None,
}

/// 无状态的数据
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Atom{
    // 实体ID
    pub ent_id: EID,
    // 属性
    pub prop_name: PropName,
    // 属性值
    pub prop_value: PropValue,
}
/// 有状态的数据条目
/// 一个Atom代表数据库中的一条
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StateAtom{
    // 记录ID，一个原子化的记录和一个Slab库绑定
    pub id: AID,
    // 无状态记录
    pub raw_atom: RwLock<Atom>,
}
impl PartialEq for StateAtom{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl StateAtom{
    fn new(id: AID, ent_id: EID, prop_name: PropName, prop_value: PropValue) -> Self{
        StateAtom{
            id,
            raw_atom: RwLock::new(Atom{
                    ent_id,
                    prop_name,
                    prop_value,
                }
            )
        }
    }
}
