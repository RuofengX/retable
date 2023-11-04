use std::ops::DerefMut;

use parking_lot::RwLock;
use bimap::BiMap;

use crate::{atom::{EID, PropValue}, PropStorage};

#[derive(Default)]
pub struct PropValueSp{
    index: RwLock<BiMap<EID, usize>>, //HACK: 使用稀疏Vec+有状态的value代替双射，牺牲一些N*usize空间换更快的速度减少两次哈希计算
    value: Vec<RwLock<PropValue>>,
}

impl PropStorage for PropValueSp{
    fn insert(&mut self, eid: EID, value: PropValue) -> Option<()> {
        let mut wtx = self.index.write();
        if wtx.contains_left(&eid){
            return None
        }
        let len = wtx.len();
        let index = len;
        wtx.insert(eid, index);
        self.value.push(RwLock::new(value));
        Some(())

    }

    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>> {
        let rtx = self.index.read();
        if let Some(r) = rtx.get_by_left(&eid){
            return Some(&self.value[*r])
        } else {
            return None
        }
    }

    fn remove(&mut self, eid: EID) -> Option<()> {
        let mut wtx = self.index.write();

        // 判断是否存在这个eid
        if let Some(&removing_index) = wtx.get_by_left(&eid){

            // 判断删除的eid是否指向了最后一个value
            let last_index = self.value.len() - 1;
            if last_index == removing_index{
                // 直接删除最后一个value和对应的index
                self.value.pop();
                wtx.remove_by_left(&eid);
                Some(())
            } else {
                // 对数组交换删除，对双射索引编辑
                self.value.swap_remove(removing_index); // last调换位置，removing_index删除

                let &last_eid = wtx.get_by_right(&last_index).unwrap();
                wtx.insert(last_eid, removing_index);
                wtx.remove_by_left(&eid);


                Some(())
            }


        } else {
            None
        }



    }

    fn tick<F>(&mut self, mut f: F)
    where 
        F: FnMut(&mut PropValue) -> () {
            self.value.iter_mut()
            .map(|value|value.write())
            .for_each(|mut wtx|f(wtx.deref_mut()));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prop_value_sp() {
        let mut prop_value_sp = PropValueSp::default();
        
        // Insert values
        assert_eq!(prop_value_sp.insert(EID(1), PropValue::Str("Value 1".to_string())), Some(()));
        assert_eq!(prop_value_sp.insert(EID(2), PropValue::Str("Value 2".to_string())), Some(()));
        assert_eq!(prop_value_sp.insert(EID(3), PropValue::Str("Value 3".to_string())), Some(()));
        assert_eq!(prop_value_sp.insert(EID(3), PropValue::Str("Value 3".to_string())), None);
        
        // Get values
        assert_eq!(prop_value_sp.get(EID(1)).map(|v| v.read().clone()), Some(PropValue::Str("Value 1".to_string())));
        assert_eq!(prop_value_sp.get(EID(2)).map(|v| v.read().clone()), Some(PropValue::Str("Value 2".to_string())));
        assert_eq!(prop_value_sp.get(EID(3)).map(|v| v.read().clone()), Some(PropValue::Str("Value 3".to_string())));
        assert_eq!(prop_value_sp.get(EID(4)).map(|v| v.read().clone()), None);
        
        // Remove values
        assert_eq!(prop_value_sp.remove(EID(3)), Some(()));
        assert_eq!(prop_value_sp.remove(EID(1)), Some(()));
        assert_eq!(prop_value_sp.remove(EID(2)), Some(()));
        assert_eq!(prop_value_sp.remove(EID(4)), None);
        
        // Verify values are removed
        assert!(prop_value_sp.get(EID(1)).is_none());
        assert!(prop_value_sp.get(EID(2)).is_none());
        assert!(prop_value_sp.get(EID(3)).is_none());
    }
}

