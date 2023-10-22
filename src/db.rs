use std::{fs::{self, File}, hash, error, fmt, path::Path, io, os::windows::prelude::FileExt};

use slab::Slab;
use multi_index_map::MultiIndexMap;
use bson::Bson;
use serde::{Serialize, Deserialize};

#[derive(MultiIndexMap, Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Atom{
    // 实体ID
    #[multi_index(ordered_non_unique)]
    pub ent_id: u32,

    // 属性
    #[multi_index(hashed_non_unique)]
    pub prop_name: String,

    // 属性值
    pub value: Bson,

    // 根据实体ID和属性计算出的索引
    #[multi_index(hashed_unique)]
    index: AtomIndex,
}
impl PartialEq for Atom{
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl Atom{
    pub fn new(ent_id: u32, prop_name: String, prop: Bson) -> Self{
        Atom{
            ent_id,
            prop_name: prop_name.clone(),
            value: prop,
            index: AtomIndex(ent_id, prop_name)
        }
    }
}

// Atom的索引
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct AtomIndex(u32, String);
impl Eq for AtomIndex{}
impl PartialEq for AtomIndex{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
// 将会根据底层的u32和String生成一个哈希值
// 也就是说，相同的u32和String生成的索引是一致的
impl hash::Hash for AtomIndex{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
} 

/// 双索引<实体-属性>数据库
pub struct Database{
    atoms: MultiIndexAtomMap, 
}
impl Database{
    pub fn new() -> Self{
        Self { atoms: MultiIndexAtomMap::default() }
    }

    /// 插入类方法
    
    /// 插入未构造的条目
    pub fn insert(&mut self, ent_id: u32, prop_name: String, prop: Bson) -> Result<(), Error>{
        let a = Atom::new(ent_id, prop_name, prop);
        self.insert_atom(a)
    }
    /// 插入已构造的条目
    pub fn insert_atom(&mut self, atom: Atom) -> Result<(), Error>{
        if let Some(_) = self.atoms.get_by_index(&atom.index){
            Err(Error::IndexAlreadyExist(&"已存在相同的<实体-属性>索引"))
        } else {
            self.atoms.insert(atom);
            Ok(())
        }
        
    }

    /// 使用属性名获取条目
    /// 没有找到时返回空列表
    pub fn get_props(&self, entity_id: u32) -> Vec<&Atom>{
        self.atoms.get_by_ent_id(&entity_id)
    }
    /// 使用实体ID获取条目
    /// 没有找到时返回空列表
    pub fn get_entities(&self, prop_name: String) -> Vec<&Atom>{
        self.atoms.get_by_prop_name(&prop_name)
    }
    pub fn get(&self, entity_id: u32, prop_name: String) -> Option<&Atom>{
        let index = AtomIndex(entity_id, prop_name);
        self.atoms.get_by_index(&index)
    }

    /// 使用<实体-属性>双索引更新属性
    /// 如果更新成功则返回被更新的Atom
    pub fn update(&mut self, entity_id: u32, prop_name: String, prop: Bson) -> Option<&Atom>{
        let index = AtomIndex(entity_id, prop_name);
        self.atoms.update_by_index(
            &index, 
            |value: &mut Bson| {
                println!("{}", value);
                *value = prop;
            }
        )
    }

    /// 将文件保存到本地
    /// TODO MultiIndexMap支持序列化后则使用原生序列化
    pub fn save(&self, path: &Path) -> Result<(), Error>{


        let s = self.atoms._store.clone();
        if let Ok(data) = ron::to_string(&s){
            let file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)?;
            file.seek_write(data.as_bytes(), 0).expect("写文件时出错");
            Ok(())
        } else {
            Err(Error::SerializeError("序列化数据时出现错误"))
        }
    }

    /// 从本地读取文件
    pub fn load(path: &Path) -> Result<Self, Error>{
        let f = fs::OpenOptions::new()
            .read(true)
            .open(path)?;

        if let Ok(data) = bincode::deserialize_from::<File, Slab<Atom>>(f){
            let _ = data;
            todo!()
        }else{
            return Err(Error::DeserializeError(&"解码数据时出错"))
        }

    }
}

#[derive(Debug)]
pub enum Error{
    IndexAlreadyExist(&'static str),
    IOError(&'static str),
    SerializeError(&'static str),
    DeserializeError(&'static str),
}
impl From<io::Error> for Error{
    fn from(_value: io::Error) -> Self {
        Self::IOError(&"输入/输出错误")
    }
}
impl fmt::Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl error::Error for Error{}
