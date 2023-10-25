use std::{sync::RwLock, collections::HashMap};

use shared_slab::Slab;

use crate::atom::{PropValue, AID, EID};

struct PropPool{
    pool: Slab<RwLock<PropValue>>,
    index: RwLock<HashMap<EID, usize>>,
}
impl PropPool{
    pub fn get(&self, eid: EID) -> Option<&RwLock<PropValue>>{

        let index = self.index.read().unwrap();
        if let Some(uid) = index.get(&eid){
            let rtx = self.pool;
            return self.pool.get(*uid)
        } else {
            None
        }
    }
    pub fn insert(&self, eid: EID, value: PropValue){

        let index = self.index.write().unwrap();
        TODO
    }
}