use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::{atom::{EID, PropValue}, PropStorage};

pub struct PropValueSp{
    index: RwLock<FxHashMap<EID, usize>>,
    value: Vec<RwLock<PropValue>>,
}

impl PropValueSp{
    pub fn new() -> Self{
        Self { index: RwLock::new(FxHashMap::default()),
            value: Vec::new() }
    }
}

impl PropStorage for PropValueSp{
    fn insert(&mut self, eid: EID, value: PropValue) -> Option<()> {
        { 
            if self.index.read().contains_key(&eid){
                return None
            }
        }
        let mut wtx = self.index.write();
        let len = wtx.len();
        wtx.insert(eid, len);
        self.value.push(RwLock::new(value));
        Some(())

    }

    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>> {
        let rtx = self.index.read();
        if let Some((_, &v)) = rtx.get_key_value(&eid){
            return Some(&self.value[v])
        } else {
            return None
        }
    }

    fn remove(&mut self, eid: EID) -> Option<()> {
        let mut wtx = self.index.write();
        let index = wtx.remove(&eid);
        if index.is_none(){
            return None
        }

        let index = index.unwrap();
        self.value.swap_remove(index);
        Some(())

    }
}
