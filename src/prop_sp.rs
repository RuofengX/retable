use std::ops::DerefMut;

use parking_lot::RwLock;

use crate::{atom::{EID, PropValue}, PropStorage};

struct StatePropValue{
    ioi: usize,  // index of this entry in PropValueSp.index
    data: RwLock<PropValue>,
}
impl StatePropValue{
    fn new(eid: EID, value: PropValue) -> Self{
        Self { ioi: eid.0, data: RwLock::new(value)}
    }
}

#[derive(Default)]
pub struct PropValueSp{
    index: RwLock<Vec<Option<usize>>>, //HACK: 使用稀疏Vec+有状态的value代替双射，牺牲一些N*usize空间换更快的速度减少两次哈希计算
    value: Vec<StatePropValue>,
}

impl PropStorage for PropValueSp{
    fn append(&mut self, eid: EID, value: PropValue) -> (){
        let mut wtx = self.index.write();

        // 先将index长度使用None补齐至eid - 1
        let current_len = wtx.len();
        let target_len = eid.0 + 1;
        if target_len > current_len{
            (current_len..(target_len-1))
            .for_each(|_|{
                wtx.push(None);
            })
        }

        // r然后对eid的属性进行修改
        self.value.push(StatePropValue::new(eid, value));
        wtx.push(Some(self.value.len() - 1));

        assert_eq!(eid.0 + 1, wtx.len());

        println!("{:?}", *wtx);
    }


    fn get(&self, eid: EID) -> Option<&RwLock<PropValue>> {
        let rtx = self.index.read();
        if eid.0 + 1 > rtx.len(){
            // 越界和为空都是返回None
            return None //由于只获取了读取锁，所以这里不处理index延长
        }
        if let Some(r) = rtx[eid.0]{
            Some(&self.value[r].data)
        } else {
            // 越界和为空都是返回None
            None
        }
    }

    fn remove(&mut self, eid: EID) -> Option<()> {
        let mut wtx = self.index.write();
        if eid.0 + 1 > wtx.len(){
            return None
        }

        let removing_index = wtx[eid.0];

        // 判断是否存在属性
        if removing_index.is_none(){
            return None
        }
        let removing_index = removing_index.unwrap();

        // 判断删除的eid是否指向了最后一个value
        let last_index = self.value.len() - 1;
        if last_index == removing_index{
            // 直接删除最后一个value
            self.value.pop();
            // 将对应的index置为None
            wtx[eid.0] = None;
            Some(())
        } else {
            let last_ioi = self.value.last().unwrap().ioi; //这里数组必不为空所以可以unwrap，这里还自动copy了ioi

            // 对数组交换删除
            self.value.swap_remove(removing_index); // last调换位置，removing_index删除

            // 对索引编辑
            wtx[last_ioi] = Some(removing_index); // 把被交换的index赋予指向最后一个数值的索引（ioi位置）
            wtx[eid.0] = None;
            Some(())
        }
    }

    fn tick<F>(&mut self, mut f: F)
    where F: FnMut(&mut PropValue) -> () {
        self.value.iter_mut()
        .for_each(|value|{
            //获取写入锁
            let mut wtx =  value.data.write();
            //执行闭包f
            f(wtx.deref_mut());
        }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prop_value_sp() {
        let mut prop_value_sp = PropValueSp::default();
        
        // Insert values
        assert_eq!(prop_value_sp.append(EID(1), PropValue::Str("Value 0".to_string())), ());
        assert_eq!(prop_value_sp.append(EID(2), PropValue::Str("Value 1".to_string())), ());
        assert_eq!(prop_value_sp.append(EID(3), PropValue::Str("Value 2".to_string())), ());
        assert_eq!(prop_value_sp.append(EID(5), PropValue::Str("Value 5".to_string())), ());
        
        // Get values
        assert_eq!(prop_value_sp.get(EID(1)).map(|v| v.read().clone()), Some(PropValue::Str("Value 0".to_string())));
        assert_eq!(prop_value_sp.get(EID(2)).map(|v| v.read().clone()), Some(PropValue::Str("Value 1".to_string())));
        assert_eq!(prop_value_sp.get(EID(3)).map(|v| v.read().clone()), Some(PropValue::Str("Value 2".to_string())));
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

