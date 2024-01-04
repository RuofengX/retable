//! A in-memory key-value store.
//!
//! Designed to be used in multi-threaded environment with high concurrency,
//! like game.
//!

pub mod binlog;
mod merge;
pub mod prop;
mod slots;

pub use prop::Prop;
