//! The core module
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{collections::BTreeMap, sync::Arc};

use crate::{
    api::{AtomStorage, PropStorage},
    basic::{Delta, Value, EID},
    error::Error,
    method::{MergeFn, TickFn},
};

use moka::sync::Cache;
pub use sled::Config;
use sled::Db;
use typed_sled::Tree;

/// As the name says.
pub struct Database {
    db: Db,
    props: BTreeMap<&'static str, Arc<dyn PropStorage>>,
}

impl Database {
    /// Create a new database.
    ///
    /// Using sled config. See more in [`Config`]
    pub fn new(conf: Config) -> Result<Self, Error> {
        Ok(Database {
            db: conf.open()?,
            props: BTreeMap::default(),
        })
    }
}

impl Default for Database {
    fn default() -> Database {
        Database {
            db: Config::default()
                .path("db/default")
                .cache_capacity(1_000_000_000)
                .flush_every_ms(Some(1000))
                .open()
                .expect("Error when open default db"),
            props: BTreeMap::default(),
        }
    }
}

impl AtomStorage for Database {
    fn get_prop(&self, prop: &'static str) -> Option<Arc<dyn PropStorage>> {
        self.props.get(prop).map(|x| x.clone())
    }

    fn create_prop<'s>(
        &'s mut self,
        prop: &'static str,
        merge: MergeFn,
        tick: TickFn,
    ) -> Arc<dyn PropStorage> {
        let prop = self
            .props
            .entry(prop)
            .or_insert_with(|| Arc::new(Prop::new(&self.db, prop, tick, merge)))
            .clone();
        prop
    }
}

/// The level-2 storage beneath Database, which impl [`crate::api::PropStorage`].
///
/// It returned by [`Database::create_prop`] or [`Database::get_prop`].
pub struct Prop<'p> {
    name: &'p str,
    tree: Tree<EID, Value>,
    tick_method: TickFn,
    cache: Cache<EID, Option<Value>>,
}
impl<'p> Prop<'p> {
    /// Create a new Prop.
    ///
    /// Note that the merge method is necessary,
    /// if not used, just invoke an empty closure like `|_,_,_|None`.
    pub fn new(db: &Db, name: &'p str, tick: TickFn, merge: MergeFn) -> Self {
        let mut rtn = Self {
            name,
            tree: Tree::<EID, Value>::open::<&str>(db, name),
            tick_method: tick,
            cache: Cache::builder().max_capacity(1024 * 1024).build(),
        };
        rtn.register_merge(merge);
        rtn
    }
}

impl<'p> PropStorage for Prop<'p> {
    /// Return the name of this Prop.
    fn name(&self) -> &str {
        self.name
    }

    /// Get a value for a eid in Prop.
    ///
    /// # Example
    /// ```rust
    /// use retable::{Database, Config, basic::{EID, Value}, api::{AtomStorage, PropStorage}};
    ///
    /// // create a temporary database to avoid old disk file polution.
    /// let mut db = Database::new(Config::default().temporary(true)).unwrap();
    /// // create a prop with non-bound method.
    /// let prop = db.create_prop("test_int", |_, _, _| None, |_,_,_|None);
    ///
    /// // Example eid is 1.
    /// let eid = EID::new(1);
    ///
    /// // Get a non-exist value, it's a None.
    /// assert_eq!(prop.get(&eid), None);
    ///
    /// // Set a Int(8) for eid(1) and get it.
    /// prop.set(&eid, Value::Int(8), false);
    /// assert_eq!(prop.get(&eid), Some(Value::Int(8)));
    /// ```
    ///
    fn get(&self, eid: &EID) -> Option<Value> {
        // 访问缓存
        if let Some(result) = self.cache.get(&eid) {
            // 缓存命中
            return result;
        }

        // 缓存未命中
        let rtn = self
            .tree
            .get(&eid)
            .expect(format!("Error when get {:?}", &eid).as_str());
        // 更新缓存
        // 对于None值，也会缓存
        self.cache.insert(*eid, rtn);
        rtn
    }

    /// Set a value for a eid in Prop.
    /// If retrieve is true, return old value.
    ///
    /// # Example
    /// ```rust
    /// use retable::{Database, Config, basic::{EID, Value}, api::{AtomStorage, PropStorage}};
    ///
    ///
    /// // create a temporary database to avoid old disk file polution.
    /// let mut db = Database::new(Config::default().temporary(false)).unwrap();
    /// // create a prop with non-bound method.
    /// let prop = db.create_prop("test_int", |_, _, _| None, |_,_,_|None);
    ///
    /// let eid = EID::new(1);
    /// // Set a Int(8) for eid(1) and get it.
    /// let old = prop.set(&eid, Value::Int(42), true);
    /// assert_eq!(old, None);
    /// assert_eq!(prop.get(&eid), Some(Value::Int(42)));
    ///
    /// // Return the old value if retrieve is true.
    /// let old = prop.set(&eid, Value::Int(43), true);
    /// assert_eq!(old, Some(Value::Int(42)));
    /// assert_eq!(prop.get(&eid), Some(Value::Int(43)));
    ///
    /// // Always return a None if retrieve is false.
    /// let old = prop.set(&eid, Value::Int(2001), false);
    /// assert_eq!(old, None);
    /// assert_eq!(prop.get(&eid), Some(Value::Int(2001)));
    /// ```
    ///
    fn set(&self, eid: &EID, value: Value, retrieve: bool) -> Option<Value> {
        self.cache.insert(*eid, Some(value));
        let old = self
            .tree
            .insert(&eid, &value)
            .expect(format!("Error when set {:?} to {:?}", eid, value).as_str());
        if retrieve {
            old
        } else {
            None
        }
    }

    /// Remove entry from Prop.
    /// If retrieve is true, return old value.
    ///
    /// # Example
    /// ```rust
    /// use retable::{Database, Config, basic::{EID, Value}, api::{AtomStorage, PropStorage}};
    ///
    /// // create a temporary database to avoid old disk file polution.
    /// let mut db = Database::new(Config::default().temporary(true)).unwrap();
    /// // create a prop with non-bound method.
    /// let prop = db.create_prop("test_int", |_, _, _| None, |_,_,_| None);
    ///
    /// // Set a value for eid(1) and eid(2) in prop.
    /// prop.set(&EID::new(1), Value::Int(42), false);
    /// prop.set(&EID::new(2), Value::Int(42), false);
    /// // Let's remove it, and fetch the old value, just like "pop".
    /// let value = prop.remove(&EID::new(1), true);
    /// assert_eq!(value, Some(Value::Int(42)));
    ///
    /// // Remove the value without retrieve will always return a None.
    /// let value = prop.remove(&EID::new(2), false);
    /// assert_eq!(value, None);
    ///
    /// // Now we lost eid(1) and eid(2) forever.
    /// assert!(prop.get(&EID::new(1)).is_none());
    /// assert!(prop.get(&EID::new(2)).is_none());
    /// ```
    fn remove(&self, eid: &EID, retrieve: bool) -> Option<Value> {
        self.cache.insert(*eid, None);
        if let Some(v) = self
            .tree
            .remove(&eid)
            .expect(format!("Error when remove prop {:?}", eid).as_str())
        {
            if retrieve {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Register a merge function for Prop.
    /// See [Prop::merge] for more.
    fn register_merge(&mut self, f: MergeFn) -> () {
        self.tree.set_merge_operator(f); // 使用typed_sled的merge方法
    }

    /// Merge a delta to a value(index by given eid) in Prop.
    ///
    /// # Example
    /// ```rust
    /// use retable::{Database, Config, basic::{EID, Value, Delta}, api::{AtomStorage, PropStorage}};
    ///
    /// // First define a merge function,
    /// // which merge a delta value int into the old value by addition, and return the new value.
    /// //
    /// // the old one is called "old", the new one is called "delta".
    /// //
    /// // Return None if either value is not int. Return Some(Value) if both values are int.
    /// // Do nothing if old value is None.
    /// // Method signature is defined by [`crate::MergeFn`]
    /// const fn int_add_merge(_: EID, old: Option<Value>, delta: Delta) -> Option<Value> {
    ///     if let Some(old) = old {
    ///         match (old, delta) {
    ///             (Value::Int(v), Delta::Int(d)) => {
    ///                 Some(Value::Int(v + d))
    ///             }
    ///             _ => {
    ///                 None
    ///             }
    ///         }
    ///     } else {
    ///         None
    ///     }
    /// }
    ///
    /// // create a temporary database to avoid old disk file polution.
    /// let mut db = Database::new(Config::default().temporary(true)).unwrap();
    /// // create a prop with int_add_merge and non-tick method.
    /// let prop = db.create_prop("test_int", int_add_merge, |_,_,_| None);
    ///
    /// // Set some value first.
    /// prop.set(&EID::new(1), Value::Int(42), false);
    /// prop.set(&EID::new(2), Value::Int(2023), false);
    ///
    /// // The delta that should be merged
    /// let delta = Delta::Int(666);
    /// // Merge it.
    /// // Note that the prop is immutable, the Sync and Send traits is garenteed by inner design.
    /// // So **DO NOT** use a Mutex (or any other lock) to protect the prop.
    /// prop.merge(&EID::new(1), delta);
    /// prop.merge(&EID::new(2), delta);
    ///
    /// // Check the result.
    /// assert_eq!(prop.get(&EID::new(1)), Some(Value::Int(708)));
    /// assert_eq!(prop.get(&EID::new(2)), Some(Value::Int(2689)));
    ///
    /// // The None value merged function is fully defined by the Merge Function
    /// // In our scenery, it will do nothing. You can modify the merge method to implement more complex logic.
    /// prop.merge(&EID::new(3), delta);
    /// assert_eq!(prop.get(&EID::new(3)), None);
    /// ```
    ///
    fn merge(&self, eid: &EID, delta: Delta) -> () {
        let new = self.tree.merge(&eid, &delta).unwrap();
        self.cache.insert(*eid, new);
    }

    /// See more in [`crate::method::TickFn`]
    fn register_tick(&mut self, f: TickFn) -> () {
        self.tick_method = f;
    }

    /// Tick all entity.
    /// 
    /// Tick actions is launched in parellar, by calling tick method using (&EID, Value, &Self)
    /// The result delta of every tick is auto merged.
    /// 
    /// See more in [`crate::method::TickFn`]
    fn tick(&self) {
        self.tree.iter().par_bridge().for_each(|i| {
            if let Ok((eid, value)) = i {
                let result = (self.tick_method)(&eid, value, self);
                if let Some(delta) = result {
                    self.merge(&eid, delta);
                }
            }
        })
    }
}
