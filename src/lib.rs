pub mod atom;
pub mod db;
mod prop_slab;
pub mod prob_sp;
use atom::{PropValue, EID};
pub use db::Props;
use parking_lot::RwLock;

trait PropStorage{
    fn insert(&mut self, eid: EID, value: PropValue) -> Option<()>;
    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>>;
    fn remove(&mut self, eid: EID) -> Option<()>;
}
