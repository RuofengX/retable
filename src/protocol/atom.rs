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

    pub const fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
}
