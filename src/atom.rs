use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct ID(u64);

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub enum PropName{
    #[default]
    None,
    DataLoss,
    Pos,
    Grid,
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, PartialOrd)]
pub enum PropValue{
    #[default]
    Loss,
    Int(i64),
    UInt(u64),
    Str(String),
    Vec([f64;3]),
    IntVec([i64;3]),
    Document(Box<PropValue>),
}
/// 无状态的数据
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RawAtom{
    // 实体ID
    pub ent_id: ID,
    // 属性
    pub prop_name: PropName,
    // 属性值
    pub prop_value: PropValue,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Atom{
    // 记录ID，一个原子化的记录和一个Slab库绑定
    pub id: usize,
    // 无状态记录
    pub raw_atom: RawAtom,
}
impl PartialEq for Atom{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Atom{
    fn new(id: usize, ent_id: ID, prop_name: PropName, prop_value: PropValue) -> Self{
        Atom{
            id,
            raw_atom: RawAtom{
                ent_id,
                prop_name,
                prop_value,
            }
        }
    }
}
