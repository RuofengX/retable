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

use parking_lot::RwLock;

pub trait PropStorage: Default{
    fn insert(&mut self, eid: EID, value: PropValue) -> Option<()>;  // 不覆盖
    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>>;
    fn remove(&mut self, eid: EID) -> Option<()>;

    // 对每个属性tick
    fn tick<F>(&mut self, f: F)
    where 
        F: FnMut(&mut PropValue) -> ();
}
