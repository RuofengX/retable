/// Merge function
///
/// # Arguments
/// * `value`: the inner mutable value to be merged
/// * `delta`: the other delta value to be merged
///
/// # Return
/// * `Some(())` if the value is merged
/// * `None` if the value is tended to be delete
pub trait MergeFn<V, D>: Fn(Option<&mut V>, D) -> bool {}
impl<V, D, T> MergeFn<V, D> for T where T: Fn(Option<&mut V>, D) -> bool {}
