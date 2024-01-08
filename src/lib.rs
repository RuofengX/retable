//! A in-memory key-value store.
//!
//! Designed to be used in multi-threaded environment with high concurrency,
//! like game.
//!

mod merge;
pub mod protocol;
pub mod dense;
pub mod persist;
