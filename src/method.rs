//! Define stateful method that interact with database.

use crate::{
    basic::{Delta, Value, EID},
    Prop,
};
use typed_sled::MergeOperator;

/// The merge method for one [crate::api::PropStorage].
///
/// It is designed to apply the change to a value.  
/// Note that the sled database require the function parameter EID needs its ownship. Since it's Copy, it won't take some difficulty to convert but ugly.
///
/// # Args
/// - `eid: EID`: Target EID, useful if the merge method is mutable (like FnMut closure).
/// - `old: Option<Value>`: The value that exists in database before merge. Option::None if the value does not exist.
/// - `delta: Delta`: The incoming delta, which needs to be merged with the current value.
///
/// # Return
/// - `Option<Value>`: The new value after merge. Option::None if the value is deleted.
///
pub trait MergeFn: Fn(EID, Option<Value>, Delta) -> Option<Value> + 'static {}
impl<T> MergeFn for T where T: MergeOperator<EID, Value> + 'static {}

/// Tick method
///
/// Designed to apply the change to a value.
///
/// # Behavior
/// Let's say if there were X number of entity in a prop.
/// This method will be called to every entity by its eid, value and the &prop, X times in total.
/// Different method calls may happen at same time.
///
/// ## Stateful
/// This method is considered as stateful, here are some interesting ideas that could be used inside a tick:
///
/// * initiate once stateful container
/// * hold a thread pool to split every tick call into different thread, the call on every method is treated as trigger
/// * use zero-mq to communicate between different tick method.
///
/// ## Thread Safe
/// Notice that, the TickFn will be used as Sync + Send, so it is your job to ensure the stateful data is
/// Send + Send inside the TickFn.
///
/// # Args
/// - `eid: EID`: The operation target id, useful if the tick method is mutable (like FnMut closure).
/// - `value: Value`: The value that exists in database before tick.
/// - `prop: &Prop`: The whole property storage that the value is stored in.
///
/// # Return
/// - `Option::<Delta>`: The new delta after tick. Option::None if stays the same (takes no effect).
///
pub trait TickFn: Fn(&EID, Value, &Prop) -> Option<Value> + Send + Sync + 'static {}
