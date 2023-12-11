/// 一个双键索引的、原子化的、kv数据库
use crate::{basic::PropStorage, AtomStorage, Error, MergeFn, TickFn};
use std::{collections::BTreeMap, sync::Arc};

use crate::basic::{Delta, Value, EID};

use sled::{Config, Db};
use typed_sled::Tree;

pub struct Database {
    db: Db,
    props: BTreeMap<&'static str, Arc<dyn PropStorage>>,
    //TODO: 添加LRU缓存
}

impl Database {
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
        tick: TickFn,
        merge: MergeFn,
    ) -> Arc<dyn PropStorage> {
        let prop = self
            .props
            .entry(prop)
            .or_insert_with(|| Arc::new(Prop::new(&self.db, prop, tick, merge)))
            .clone();
        prop
    }
}

pub struct Prop<'p> {
    name: &'p str,
    tree: Tree<EID, Value>,
    tick_method: TickFn,
}
impl<'p> Prop<'p> {
    pub fn new(db: &Db, name: &'p str, tick: TickFn, merge: MergeFn) -> Self {
        let mut rtn = Self {
            name,
            tree: Tree::<EID, Value>::open::<&str>(db, name),
            tick_method: tick,
        };
        rtn.register_merge(merge).unwrap();
        rtn
    }
}

impl<'p> PropStorage for Prop<'p> {
    fn name(&self) -> &str {
        self.name
    }
    fn get(&self, eid: EID) -> Option<Value> {
        let k = self
            .tree
            .get(&eid)
            .expect(format!("Error when get {:?}", &eid).as_str());
        match k {
            Some(v) => Some(v),
            None => None,
        }
    }

    fn set(&self, eid: EID, value: Value, retrieve: bool) -> Option<Value> {
        if let Some(v) = self
            .tree
            .insert(&eid, &value)
            .expect(format!("Error when set {:?} to {:?}", eid, value).as_str())
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

    fn remove(&self, eid: EID, retrieve: bool) -> Option<Value> {
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

    fn register_merge(&mut self, f: MergeFn) -> Result<(), Error> {
        self.tree.set_merge_operator(f); // 使用typed_sled的merge方法
        Ok(())
    }

    fn merge(&self, eid: EID, delta: Delta) -> () {
        self.tree.merge(&eid, &delta).expect("没有注册merge函数");
    }

    fn register_tick(&mut self, f: TickFn) -> Result<(), Error> {
        self.tick_method = f;
        Ok(())
    }

    fn tick(&self) {
        // 存在prop属性的bucket
        for i in self.tree.iter() {
            // 遍历bucket
            if let Ok((eid, value)) = i {
                // 成功获取eid和value
                if let Some(result) = (self.tick_method)(eid, value, self) {
                    // 成功调用tick方法
                    let _ = self.tree.merge(&eid, &result); // 合并结果
                }
            }
        }
    }
}
