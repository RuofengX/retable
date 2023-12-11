pub mod atom;
pub mod basic;
pub mod db;
pub mod error;

pub use basic::{
    method::{MergeFn, TickFn},
    AtomStorage, Delta, PropStorage, Value, EID,
};
pub use db::Database;
pub use error::Error;
pub use sled::Config;
