//! A in-memory key-value store.
//!
//! Designed to be used in multi-threaded environment with high concurrency,
//! like game.
//!

#[cfg(feature="binlog")]
pub mod binlog;
mod merge;
pub mod prop;
mod slots;
mod datalayer;

pub use prop::Prop;
