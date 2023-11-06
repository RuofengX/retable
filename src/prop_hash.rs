use std::ops::DerefMut;

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{atom::{PropValue, EID}, PropStorage};

#[derive(Default, Deserialize, Serialize)]
pub struct PropValueHash(FxHashMap<EID, RwLock<PropValue>>);
impl PropStorage for PropValueHash{

    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>> {
        self.0.get(&eid)
    }

    fn append(&mut self, eid:EID, value: PropValue) -> (){
        self.0.insert(eid, RwLock::new(value));
    }

    fn remove(&mut self, eid: EID) -> Option<()> {
        if self.0.remove(&eid).is_some(){
            return Some(())
        }
        None
    }

    fn tick<F>(&mut self, mut f: F)
    where 
        F: FnMut(&mut PropValue) -> () {
            self.0.values_mut()
            .map(|value|value.write())
            .for_each(|mut wtx|f(wtx.deref_mut()));
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prop_value_sp() {
        let mut prop_value_sp = PropValueHash::default();
        
        // Insert values
        assert_eq!(prop_value_sp.append(EID(1), PropValue::Str("Value 0".to_string())), ());
        assert_eq!(prop_value_sp.append(EID(2), PropValue::Str("Value 1".to_string())), ());
        assert_eq!(prop_value_sp.append(EID(3), PropValue::Str("Value 2".to_string())), ());
        assert_eq!(prop_value_sp.append(EID(5), PropValue::Str("Value 5".to_string())), ());
        
        // Get values
        assert_eq!(prop_value_sp.get(EID(1)).map(|v| v.read().clone()), Some(PropValue::Str("Value 0".to_string())));
        assert_eq!(prop_value_sp.get(EID(2)).map(|v| v.read().clone()), Some(PropValue::Str("Value 1".to_string())));
        assert_eq!(prop_value_sp.get(EID(3)).map(|v| v.read().clone()), Some(PropValue::Str("Value 2".to_string())));
        assert_eq!(prop_value_sp.get(EID(4)).map(|v| v.read().clone()), None);
        assert_eq!(prop_value_sp.get(EID(5)).map(|v| v.read().clone()), Some(PropValue::Str("Value 5".to_string())));
        
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

