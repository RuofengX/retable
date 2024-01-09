use std::ops::AddAssign;
use std::hash::Hash;

use zerocopy::{AsBytes, FromBytes, FromZeroes};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes, Unaligned};

/// Merge D into V(Self)
pub trait MergeAssign {
    type Delta;
    fn merge(&mut self, delta: Self::Delta);
}

/// An default impl of MergeAssign
impl<T: AddAssign> MergeAssign for T {
    type Delta = T;
    fn merge(&mut self, delta: T) {
        *self += delta
    }
}

pub trait Atomic: Sized {
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
    fn get_persist(&self) -> &impl LogWriter<Self::K, Self::V, Self::D>;

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
                    self.get_persist().save_one(Atom::new(
                        UPDATE,
                        *key,
                        value.clone(),
                        Self::D::new_zeroed(),
                    ));
                }

                unsafe { Some(self.update_unchecked(key, value)) }
            } else {
                // old value not exists, create new, return none

                // handle persist
                #[cfg(feature = "persist")]
                {
                    self.get_persist().save_one(Atom::new(
                        CREATE,
                        *key,
                        value.clone(),
                        Self::D::new_zeroed(),
                    ));
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
                self.get_persist().save_one(Atom::new(
                    DELETE,
                    *key,
                    Self::V::new_zeroed(),
                    Self::D::new_zeroed(),
                ));
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
                self.get_persist().save_one(Atom::new(
                    MERGE,
                    *key,
                    Self::V::new_zeroed(),
                    delta.clone(),
                ));
            }
            unsafe { self.merge_unchecked(key, delta) }
        }
    }

    /// load from an Iterator of atoms.
    fn load(&self, from: impl Iterator<Item = Atom<Self::K, Self::V, Self::D>>) {
        from.into_iter().for_each(|atom| {
            let (op, k, v, d) = atom.into_align();
            match op {
                CREATE => unsafe {
                    self.create_unchecked(&k, &v);
                },
                UPDATE => unsafe {
                    self.update_unchecked(&k, &v);
                },
                MERGE => unsafe {
                    self.merge_unchecked(&k, &d);
                },
                DELETE => unsafe {
                    self.delete_unchecked(&k);
                },
                _ => {}
            }
        });
    }
}

/// Operation hint for atomic trait.
///
/// Atomic storage protocol only record 4 ops below.
pub type OperationHint = u8;

// Create a key-value pair which does not exist before.
pub const CREATE: OperationHint = 0;

// Update an exist key-value pair.
pub const UPDATE: OperationHint = 1;

// Merge a value. Atom do not include the delta funtion.
pub const MERGE: OperationHint = 2;

// Delete a value.
pub const DELETE: OperationHint = 3;

#[derive(Debug, AsBytes, FromBytes, FromZeroes, Unaligned)]
#[repr(packed)]
pub struct Atom<K, V, D> {
    op: OperationHint,
    key: K,
    value: V,
    delta: D,
}
impl<K, V, D> Atom<K, V, D> {
    pub const fn len(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    pub fn new(op: OperationHint, key: K, value: V, delta: D) -> Self {
        Atom {
            op,
            key,
            value,
            delta,
        }
    }

    #[inline]
    pub fn into_align(self) -> (OperationHint, K, V, D) {
        (self.op, self.key, self.value, self.delta)
    }
}

/// Wrapper for IO object
pub trait LogWriter<K, V, D> {
    fn save_one(&self, data: Atom<K, V, D>);
}

/// A nothing-to-do log writer.
impl<K, V, D> LogWriter<K, V, D> for () {
    /// Must be unblocked
    fn save_one(&self, _data: Atom<K, V, D>) {}
}

pub trait LogReader<'a, K, V, D>
where
    K: 'a,
    V: 'a,
    D: 'a,
{
    fn iter(&self) -> impl Iterator<Item = &'a Atom<K, V, D>>;
    fn into_iter(&self) -> impl Iterator<Item = Atom<K, V, D>>;
}
