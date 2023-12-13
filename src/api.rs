//! A set of trait that defines the behavior of database and property storage.
use std::sync::Arc;

use crate::{
    basic::{Delta, PropTag, Value, EID},
    method::{MergeFn, TickFn},
};

/// A trait that defines the behavior of property storage.
///
/// 'prop' means that the storage is only for one property.
pub trait PropStorage: Sync + Send {
    /// Get name of prop.
    fn name(&self) -> PropTag;

    /// Get entity's value. None if not exists.
    ///
    /// Thread safe.
    fn get(&self, eid: &EID) -> Option<Value>;

    /// Set a value for entity.
    ///
    /// Return the raw value if retrieve is true.
    /// Return None if retrieve is false.
    ///
    /// Thread safe.
    fn set(&self, eid: &EID, value: Value, retrieve: bool) -> Option<Value>;

    /// Delete a entity's value.
    ///
    /// Return the raw value if retrieve is true.
    /// Return None if retrieve is false.
    ///
    /// Thread safe.
    fn remove(&self, eid: &EID, retrieve: bool) -> Option<Value>;

    /// Register merge method.
    /// Always cover the old one.
    ///
    /// See more in [`crate::method::MergeFn`]
    fn register_merge(&mut self, f: MergeFn) -> ();

    /// Call the merge function to merge a Delta(alias for Value) into an exist value.
    ///
    /// The behavior is defined by the MergeFn registed when eid is not exist.
    /// A PropStorage always holds a merge method.
    fn merge(&self, eid: &EID, delta: Delta) -> ();

    /// Register prop-level tick method.
    /// Always cover the old one.
    ///
    /// See more in [`crate::method::TickFn`]
    fn register_tick(&mut self, f: TickFn) -> ();

    /// Call the tick function to update the whole prop.
    ///
    /// Will call the tick function for every
    /// registed when eid is not exist.
    /// A PropStorage always holds a merge method.
    fn tick(&self);

    // TODO: 添加批量merge操作
    // TODO: 添加输入、输出流
    // TODO: 添加默认的merge函数
}

/// The trait that design for database storage.
pub trait AtomStorage {
    /// Get Prop reference from Database.
    ///
    /// Return None if not exist.
    fn get_prop(&self, prop: &PropTag) -> Option<Arc<dyn PropStorage>>;

    /// If already exists, return the old data but register new method.
    fn create_prop(&mut self, prop: PropTag, merge: MergeFn, tick: TickFn) -> Arc<dyn PropStorage>;
}
