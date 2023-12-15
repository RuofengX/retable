//! ATOM, Atomic Type Object Model

use std::hash::Hash;
use serde::{Deserialize, Serialize};


pub trait Key: Clone + Copy + Eq + Ord + Hash + AsRef<[u8]> + Send + Sync {}
pub trait Value: AsRef<[u8]> + Default {}
pub trait Delta: AsRef<[u8]> {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Atom<K, V>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    pub eid: usize,
    pub prop: K,
    pub value: V,
}
