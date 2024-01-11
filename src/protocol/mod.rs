mod atomic;
pub use atomic::{Atomic, LogReader, LogWriter};
use std::ops::AddAssign;

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
