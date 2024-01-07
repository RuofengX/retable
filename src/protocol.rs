use zerocopy::{AsBytes, FromBytes, FromZeroes};
use zerocopy_derive::{AsBytes, FromBytes, FromZeroes, Unaligned};

pub trait MergeAssign<D> {
    fn merge(&mut self, delta: D);
}

pub trait Atomic: AsBytes + FromBytes + FromZeroes + Sized {
    /// The key type. For index usage.
    type K: Ord + Copy + AsBytes + FromBytes + FromZeroes;
    /// The value type.
    type V: Default + Clone + MergeAssign<Self::D> + AsBytes + FromBytes + FromZeroes;
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
    unsafe fn update_unchecked(&self, key: &Self::K) -> Self::V;

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

    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }

    fn zmq_endpoint() -> String {
        format!("inproc://atom/archive/{}", std::any::type_name::<Self>())
    }

    /// Return the beans of zmq::Context.
    fn zmq_ctx(&self) -> &zmq::Context;

    /// Return the beans of zmq::Socket.
    fn zmq_sock(&self) -> &zmq::Socket;

    /// Return the mutable beans of zmq::Socket.
    fn zmq_sock_mut(&mut self) -> &mut zmq::Socket;

    fn zmq_init(&mut self) -> Result<(), zmq::Error> {
        *self.zmq_sock_mut() = self.zmq_ctx().socket(zmq::PULL)?;
        self.zmq_sock_mut().bind(&Self::zmq_endpoint())?;
        Ok(())
    }

    fn zmq_send(&self, msg: &Atom<Self::K, Self::V, Self::D>) {
        self.zmq_sock().send(msg.as_bytes(), 0).unwrap();
    }

    fn zmq_recv(&self) -> Atom<Self::K, Self::V, Self::D> {
        let mut buf = Atom::<Self::K, Self::V, Self::D>::new_zeroed();
        self.zmq_sock()
            .recv_into(buf.as_bytes_mut(), zmq::DONTWAIT)
            .unwrap();
        buf
    }

    /// Check if the key exists.
    fn contains_key(&self, key: &Self::K) -> bool;

    /// Ensure an entry is created with the given value.
    ///
    /// Return an old value if overwrited.
    /// Return None if the key does not exist before.
    fn set(&self, key: &Self::K, value: &Self::V) -> Option<Self::V> {
        if self.contains_key(key) {
            unsafe { Some(self.update_unchecked(key)) }
        } else {
            unsafe {
                self.create_unchecked(key, value);
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

    /// Ensure remove an entry.
    ///
    /// Return a value if existed.
    /// Return None if the key does not exist before,
    /// also that means nothing change after this op.
    fn remove(&self, key: &Self::K) -> Option<Self::V> {
        if self.contains_key(key) {
            unsafe { Some(self.delete_unchecked(key)) }
        } else {
            None
        }
    }

    /// Save all the key-value pairs into many Atom with minimal entropy.
    fn iter(self) -> impl Iterator<Item = Atom<Self::K, Self::V, Self::D>>;

    /// load from an Iterator of atoms.
    fn load(from: impl Iterator<Item = Atom<Self::K, Self::V, Self::D>>) -> Self {
        let rtn = Self::new_zeroed();
        from.into_iter().for_each(|atom| {
            let (op, k, v, d) = atom.into_align();
            match op {
                CREATE => unsafe {
                    rtn.create_unchecked(&k, &v);
                },
                UPDATE => unsafe {
                    rtn.update_unchecked(&k);
                },
                MERGE => unsafe {
                    rtn.merge_unchecked(&k, &d);
                },
                DELETE => unsafe {
                    rtn.delete_unchecked(&k);
                },
                _ => {}
            }
        });
        rtn
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
