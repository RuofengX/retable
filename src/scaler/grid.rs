use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

/// 三维向量
#[derive(Clone, Copy, Serialize, Deserialize)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub struct IntVector3(pub [i64; 3]);

/// 四则运算
impl Add<IntVector3> for IntVector3{
    type Output = IntVector3;

    fn add(self, mut rhs: IntVector3) -> Self::Output {
        for i in 0..3 {
            rhs.0[i] += self.0[i];
        }
        rhs
    }
}
impl Sub<IntVector3> for IntVector3 {
    type Output = IntVector3;

    fn sub(self, mut rhs: IntVector3) -> Self::Output {
        for i in 0..3 {
            rhs.0[i] -= self.0[i];
        }
        rhs
    }
}

impl Mul<i64> for IntVector3 {
    type Output = IntVector3;

    fn mul(self, rhs: i64) -> Self::Output {
        let mut rtn = [0i64;3];
        for i in 0..3 {
            rtn[i] = self.0[i] * rhs;
        }
        rtn.into()
    }
}
impl Div<i64> for IntVector3 {
    type Output = IntVector3;

    fn div(self, rhs: i64) -> Self::Output {
        let mut rtn = [0i64;3];
        for i in 0..3 {
           rtn[i] = self.0[i] / rhs;
        }
        rtn.into()
    }
}

/// 类型转换
impl From<[i64; 3]> for IntVector3 {
    fn from(value: [i64; 3]) -> Self {
        IntVector3 { 0: value }
    }
}
impl IntVector3 {
    /// 三维
    pub fn x(&self) -> i64{
        self.0[0]
    }
    pub fn y(&self) -> i64 {
        self.0[1]
    }
    pub fn z(&self) -> i64 {
        self.0[2]
    }
    pub fn get_length(&self) -> f64 {
        ((self.x().pow(2) + self.y().pow(2) + self.z().pow(2)) as f64 ).sqrt()
    }

    pub fn get_length_floor(&self) -> i64{
        self.get_length().floor() as i64
    }

    pub fn get_length_ceil(&self) -> i64{
        self.get_length().ceil() as i64
    }

    // pub fn get_unit(&self) -> IntVector3 {
    //     *self / self.get_length()
    // }
    
    /// 零长度向量
    pub fn zero() -> IntVector3 {
        IntVector3([0i64; 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut v1 = IntVector3([1, 2, 3]);
        let v2 = IntVector3([4, 5, 6]);

        let v1b = &mut v1;
        *v1b = v1b.add(v2);

        assert_eq!(v1, IntVector3([5, 7, 9]));
    }
}
