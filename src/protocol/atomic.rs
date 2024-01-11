use std::hash::Hash;

use zerocopy::{AsBytes, FromBytes, FromZeroes};

use super::MergeAssign;

pub trait Atomic: Default + Sized {
    /// The key type. For index usage.
    type K: Hash + Ord + Copy + AsBytes + FromBytes + FromZeroes;
    /// The value type.
    type V: Default + Clone + MergeAssign<Delta = Self::D> + AsBytes + FromBytes + FromZeroes;
    type D: Clone + AsBytes + FromBytes + FromZeroes;

    /// Create a new entry atomically.
    ///
    /// # Safety
    /// * Assume that the key does not exist in self before.
    /// * No binlog save caller to zmq.
    unsafe fn create_unchecked(&self, key: &Self::K, value: &Self::V);

    /// Read a copy of existed entry atomically.
    ///
    /// # Safety
    /// * Assume that the key does exist in self before.
    /// * No binlog save caller to zmq.
    unsafe fn read_unchecked(&self, key: &Self::K) -> Self::V;

    /// Update a existed entry atomically.
    ///
    /// # Safety
    /// * Assume that the key does exist in self before.
    /// * No binlog save caller to zmq.
    unsafe fn update_unchecked(&self, key: &Self::K, value: &Self::V) -> Self::V;

    /// Modify a existed entry with a delta.
    ///
    /// The behavior of merge is defined by [`crate::datalayer::MergeAssign`]
    ///
    /// # Safety
    /// * Assume that the key does exist in self before.
    /// * No binlog save caller to zmq.
    unsafe fn merge_unchecked(&self, key: &Self::K, delta: &Self::D);

    /// Delete an existed entry atomically.
    ///
    /// # Safety
    /// * Assume that the key exist in self before.
    /// * No binlog save caller to zmq.
    unsafe fn delete_unchecked(&self, key: &Self::K) -> Self::V;

    #[inline]
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Check if the key exists.
    fn contains_key(&self, key: &Self::K) -> bool;

    /// Get persistence handler
    #[cfg(feature = "persist")]
    fn log_writer(&self) -> &impl LogWriter<Self::K, Self::V, Self::D>;

    /// Ensure an entry is created with the given value.
    ///
    /// Return an old value if overwrited.
    /// Return None if the key does not exist before.
    fn set(&self, key: &Self::K, value: Option<&Self::V>) -> Option<Self::V> {
        if let Some(value) = value {
            // modify the entry
            if self.contains_key(key) {
                // old value exists, swap old and new, return old

                // handle persist
                #[cfg(feature = "persist")]
                {
                    self.log_writer().update(key, value);
                }

                unsafe { Some(self.update_unchecked(key, value)) }
            } else {
                // old value not exists, create new, return none

                // handle persist
                #[cfg(feature = "persist")]
                {
                    self.log_writer().create(key, value);
                }

                unsafe {
                    self.create_unchecked(key, value);
                    None
                }
            }
        } else {
            // delete the entry

            // handle persist
            #[cfg(feature = "persist")]
            {
                self.log_writer().delete(key);
            }

            if self.contains_key(key) {
                // old value exists, take old
                unsafe { Some(self.delete_unchecked(key)) }
            } else {
                // old value not exists, return none
                None
            }
        }
    }

    // Get a value, None if not exist.
    fn get(&self, key: &Self::K) -> Option<Self::V> {
        if self.contains_key(key) {
            unsafe { Some(self.read_unchecked(key)) }
        } else {
            None
        }
    }

    fn merge(&self, key: &Self::K, delta: &Self::D) {
        if self.contains_key(key) {
            // handle persist
            #[cfg(feature = "persist")]
            {
                self.log_writer().merge(key, delta);
            }
            unsafe { self.merge_unchecked(key, delta) }
        }
    }

    /// load from an Iterator of atoms.
    fn load(from: impl Iterator<Item = (Self::K, Self::V)>) -> Self {
        let rtn = Self::default();
        from.into_iter().for_each(|(k, v)| {
            unsafe {
                rtn.create_unchecked(&k, &v);
            };
        });
        rtn
    }
}

/// Wrapper for IO object
pub trait LogWriter<K, V, D> {
    fn create(&self, key: &K, value: &V);
    fn update(&self, key: &K, value: &V);
    fn merge(&self, key: &K, delta: &D);
    fn delete(&self, key: &K);
}

/// A nothing-to-do log writer.
/// Every operation must be unblocked
impl<K, V, D> LogWriter<K, V, D> for () {
    fn create(&self, _key: &K, _value: &V) {}
    fn update(&self, _key: &K, _value: &V) {}
    fn merge(&self, _key: &K, _delta: &D) {}
    fn delete(&self, _key: &K) {}
}

pub trait LogReader<K, V>: Iterator<Item = (K, V)> {}
