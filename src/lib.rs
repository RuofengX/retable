pub mod atom;
pub mod db;
pub mod prop_sp;
pub mod prop_hash;
pub mod scaler;
pub use atom::{PropValue, EID};
pub use db::Props;
pub use prop_sp::PropValueSp;
pub use prop_hash::PropValueHash;
pub use scaler::Vector3;
use serde::Serialize;

use parking_lot::RwLock;

pub trait PropStorage: Default + Serialize{
    // 为eid新增一个属性，eid永远是新增的
    fn append(&mut self, eid:EID, value: PropValue) -> ();

    // 获取eid的属性
    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>>;

    // 删除eid的属性
    fn remove(&mut self, eid: EID) -> Option<()>;

    // 对每个属性tick
    fn tick<F>(&mut self, f: &mut F)
    where 
        F: FnMut(&mut PropValue) -> ();
}
