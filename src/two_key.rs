use std::{collections::{BTreeSet, BTreeMap}, sync::RwLock, };

use slab::Slab;

use crate::{err::Error, atom::{ID, Atom, PropName, PropValue}};


#[derive(Debug, Default)]
pub struct TwoDimensionMap{
    pool: RwLock<Slab<Atom>>,
    id_index: BTreeMap<usize, usize>,
    entity_index: BTreeMap<ID, BTreeSet<usize>>,
    prop_index: BTreeMap<PropName, BTreeSet<usize>>,
}
impl TwoDimensionMap{
    // pub fn new() -> Self{
    //     Self { pool: Slab::new(), id_index: HashMap::new(), entity_index: HashMap::new(), prop_index: HashMap::new() }
    // }
    pub fn default() -> Self{
        Self { 
        pool: RwLock::new(Slab::default()),
            id_index: BTreeMap::default(), 
            entity_index: BTreeMap::default(), 
            prop_index: BTreeMap::default()
        }
    }

    /// 检查类方法
    
    /// 检查是否包含记录
    pub fn check_id(&self, id: usize) -> bool{
        self.id_index.contains_key(&id)
    }
    /// 检查是否包含实体
    pub fn check_entity(&self, ent_id: ID) -> bool{
        self.entity_index.contains_key(&ent_id)
    }
    /// 检查是否包含属性
    pub fn check_prop(&self, prop: PropName) -> bool{
        self.prop_index.contains_key(&prop)
    }

    /// 插入类方法
    
    TODO: Here
    /// 插入未构造的条目
    pub fn insert(&mut self, ent_id: ID, prop: PropName, value: PropValue) -> Result<usize, Error>{
        let a = Atom{ent_id, prop_name: prop, prop_value: value };
    }
    /// 插入已构造的条目
    pub fn insert_atom(&mut self, atom: Atom) -> Result<usize, Error>{

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
                .truncate(true)
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


        if let Ok(data) = ron::de::from_reader::<File, Slab<Atom>>(f){
            let mut rtn = Self::new();
            for (_, atom) in data.iter(){
                rtn.insert_atom(atom.clone())?;
            }
            Ok(rtn)
        }else{
            Err(Error::DeserializeError(&"解码数据时出错"))
        }

    }
}
