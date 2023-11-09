pub mod atom;
pub mod db;
pub mod prop_hash;
pub mod prop_sp;
pub mod scaler;
pub use atom::{PropValue, EID};
pub use db::Props;
pub use prop_hash::PropValueHash;
pub use prop_sp::PropValueSp;
pub use scaler::Vec3;
use serde::{Deserialize, Serialize};

use parking_lot::RwLock;

/// prop存储方案必须要实现的特质
pub trait PropStorage: Default + Serialize + for<'a> Deserialize<'a> {
    // 为eid新增一个属性，eid永远是新增的
    fn append(&mut self, eid: EID, value: PropValue) -> ();

    // 获取eid的属性
    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>>;

    // 删除eid的属性
    fn remove(&mut self, eid: EID) -> Option<()>;

    // 对每个属性tick
    fn tick<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut PropValue) -> ();
}

/// 一个wrapper类型，指向了底层的PropValue的存储方案
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Prop(PropValueSp);
impl PropStorage for Prop {
    fn append(&mut self, eid: EID, value: PropValue) -> () {
        self.0.append(eid, value)
    }

    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>> {
        self.0.get(eid)
    }

    fn remove(&mut self, eid: EID) -> Option<()> {
        self.0.remove(eid)
    }

    fn tick<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut PropValue) -> (),
    {
        self.0.tick(f)
    }
}
