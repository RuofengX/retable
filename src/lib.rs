pub mod atom;
pub mod db;
pub mod prob_sp;
mod prob_hash;
use atom::{PropValue, EID};
pub use db::Props;
use parking_lot::RwLock;

pub trait PropStorage{
    fn insert(&mut self, eid: EID, value: PropValue) -> Option<()>;  // 不覆盖
    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>>;
    fn remove(&mut self, eid: EID) -> Option<()>;
}
