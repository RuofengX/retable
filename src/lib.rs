//! A in-memory key-value store.
//!
//! Designed to be used in multi-threaded environment with high concurrency,
//! like game.
//! 

mod binlog;
mod merge;
mod slots;
pub mod prop;

pub use prop::Prop;
