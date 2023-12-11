use crate::basic::{Delta, Value, EID};

/// The merge method for one [crate::api::PropStorage].
///
/// It is designed to apply the change to a value.
///
/// # Args
/// - `eid: EID`: Target EID, useful if the merge method is mutable (like FnMut closure).
/// - `old: Option<Value>`: The value that exists in database before merge. Option::None if the value does not exist.
/// - `delta: Delta`: The incoming delta, which needs to be merged with the current value.
///
/// # Return
/// - `Option<Value>`: The new value after merge. Option::None if the value is deleted.
///
pub type MergeFn = fn(eid: EID, old: Option<Value>, delta: Delta) -> Option<Value>;

/// Tick method
/// Designed to apply the change to a value.
///
/// # Args
/// - `eid: EID`: The operation target id, useful if the tick method is mutable (like FnMut closure).:The storage that the value is stored in.
/// - `value: Value`: The value that exists in database before tick.
/// - `prop: &dyn PropStorage`: The whole property storage that the value is stored in.
///
/// # Return
/// - `Option::<Delta>`: The new delta after tick. Option::None if stays the same (takes no effect).
pub type TickFn = fn(eid: EID, value: Value, prop: &dyn crate::api::PropStorage) -> Option<Delta>;
