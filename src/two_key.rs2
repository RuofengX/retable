use std::{collections::{BTreeSet, BTreeMap}, sync::{RwLock, Mutex, RwLockReadGuard}, intrinsics::atomic_nand_acqrel, };

use slab::Slab;

use crate::{err::Error, atom::{AID, EID, StateAtom, Atom, PropName, PropValue}};

#[derive(Debug, Default)]
struct TwoIndex{
    id: BTreeMap<AID, usize>, // 由记录ID指向实际slab的usize，该usize可能发生变更
    ent: BTreeMap<EID, BTreeSet<AID>>,
    prop: BTreeMap<PropName, BTreeSet<AID>>,
}

#[derive(Debug, Default)]
pub struct TwoDimensionMap{
    pool: Slab<StateAtom>,
    index: RwLock<TwoIndex>,
}
impl TwoDimensionMap{
    pub const fn new() -> Self{
        TwoDimensionMap { 
            pool: Slab::new(), 
            index: RwLock::new(TwoIndex{
                id: BTreeMap::new(),
                ent: BTreeMap::new(),
                prop: BTreeMap::new(),
            }),
        }
    }

    /// 检查是否包含记录
    pub fn check_id(&self, id: AID) -> bool{
        self.index.read().unwrap().id.contains_key(&id)
    }
    /// 检查是否包含实体
    pub fn check_entity(&self, ent_id: &EID) -> bool{
        self.index.read().unwrap().ent.contains_key(ent_id)
    }
    /// 检查是否包含属性
    pub fn check_prop(&self, prop: &PropName) -> bool{
        self.index.read().unwrap().prop.contains_key(prop)
    }

    /// 插入未构造的条目
    pub fn insert(
        &mut self, 
        ent_id: EID, 
        prop: PropName, 
        value: PropValue
    ) -> Result<AID, Error>{
        let a = Atom{ent_id, prop_name: prop, prop_value: value };
        Ok(self.insert_atom(a)?)
    }
    /// 插入已构造的条目
    pub fn insert_atom(&mut self, atom: Atom) -> Result<AID, Error>{
        // 检查是否存在
        if self.check_entity(&atom.ent_id) && self.check_prop(&atom.prop_name) {
            return Err(Error::DuplicateKey(&"相同的实体-属性键已经存在，如需要修改请使用.update()"))
        }

        // 获取写入锁
        let mut wtx = self.index.write().unwrap();

        // 获取索引值
        let slab_id = self.pool.vacant_key(); // 在slab中存储的usize，可能在重分配后变化
        let id = AID::new(wtx.id.len()); // 该条记录的ID，始终不会变化

        // 插入数据到数据池
        self.pool.insert(StateAtom { id, raw_atom: RwLock::new(atom)} );

        // 插入主索引
        wtx.id.insert(id, slab_id);

        // 插入子索引
        wtx.ent.entry(atom.ent_id) // 入口
            .or_insert(BTreeSet::new()) // 返回mut entry，如不存在该键则insert一个空BTreeSet
            .insert(id); // 插入记录ID
        wtx.prop.entry(atom.prop_name)
            .or_insert(BTreeSet::new())
            .insert(id);

        Ok(id)


    }

    /// 使用属性名获取条目
    /// 没有找到时返回空列表
    pub fn get_props(&self, entity_id: &EID) -> Vec<&StateAtom>{
        let rtx = self.index.read().unwrap();
        match rtx.ent.get(&entity_id){
            Some(atom_index) => {
                atom_index.iter() // 全部的记录ID
                    .map(|aid|self.get_by_aid(aid, Some(rtx)).unwrap())  // 先转换成Slab的ID，然后再获取对应借用
                    .collect()
            },
            None => Vec::new(),
        }
    }
    /// 使用实体ID获取条目
    /// 没有找到时返回空列表
    pub fn get_entities(&self, prop_name: &PropName) -> Vec<&StateAtom>{
        let rtx = self.index.read().unwrap();
        match rtx.prop.get(prop_name){
            Some(atom_index) => {
                atom_index.iter()
                    .map(|aid|self.get_by_aid(aid, Some(rtx)).unwrap())
                    .collect()
            },
            None => Vec::new(),
        }
    }
    pub fn get(&self, entity_id: &EID, prop_name: &PropName) -> Option<&StateAtom>{
        let rtx = self.index.read().unwrap();
        match (rtx.ent.get(entity_id), rtx.prop.get(prop_name)){
            (Some(atom_index), Some(atom_index2) )=> {
                if let Some(aid) = atom_index.intersection(atom_index2).next(){
                    self.get_by_aid(aid, Some(rtx))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
    pub fn get_by_aid(&self, aid: &AID, with_rtx: Option<RwLockReadGuard<TwoIndex>>) -> Option<&StateAtom>{
        if let Some(rtx) = with_rtx{
            self.pool.get(rtx.id[aid])
        } else {
            let rtx = self.index.read().unwrap();
            self.pool.get(rtx.id[aid])
        }
    }

    /// 使用<实体-属性>双索引更新属性
    /// 如果更新成功则返回被更新的Atom
    pub fn update(&mut self, entity_id: EID, prop_name: PropName, value: PropValue) -> Option<&StateAtom>{
        let index = AID(entity_id, prop_name);
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


        if let Ok(data) = ron::de::from_reader::<File, Slab<StateAtom>>(f){
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
