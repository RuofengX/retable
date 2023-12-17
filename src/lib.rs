#![warn(missing_docs)]
//! A two key value store.
//!
//! Used by entropy-rs, The Game · Created by RuofengX.
//!
//!
//! Features:
//! * No-unsafe, also source code.
//! * Thread-safe, build on top of sled and moka.
//! * Persistent, check [`sled::Config`] to learn more about the database.
//!

pub mod atom;
pub mod basic;
pub mod db;
pub mod error;
pub mod method;

pub use db::{Database, Prop};
pub use sled::Config;
