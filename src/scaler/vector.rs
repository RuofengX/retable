use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

/// 三维向量
#[derive(Clone, Copy, Serialize, Deserialize)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub struct Vector3([f64; 3]);

/// 四则运算
impl Add<Vector3> for Vector3 {
    type Output = Vector3;

    fn add(self, mut rhs: Vector3) -> Self::Output {
        for i in 0..3 {
            rhs.0[i] += self.0[i];
        }
        rhs
    }
}
impl Sub<Vector3> for Vector3 {
    type Output = Vector3;

    fn sub(self, mut rhs: Vector3) -> Self::Output {
        for i in 0..3 {
            rhs.0[i] -= self.0[i];
        }
        rhs
    }
}

impl Mul<f64> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f64) -> Self::Output {
        let mut rtn = [0.0;3];
        for i in 0..3 {
            rtn[i] = self.0[i] * rhs;
        }
        rtn.into()
    }
}
impl Div<f64> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f64) -> Self::Output {
        let mut rtn = [0.0;3];
        for i in 0..3 {
           rtn[i] = self.0[i] / rhs;
        }
        rtn.into()
    }
}

/// 类型转换
impl From<[f64; 3]> for Vector3 {
    fn from(value: [f64; 3]) -> Self {
        Vector3 { 0: value }
    }
}
impl Vector3 {
    /// 三维
    pub fn x(&self) -> f64 {
        self.0[0]
    }
    pub fn y(&self) -> f64 {
        self.0[1]
    }
    pub fn z(&self) -> f64 {
        self.0[2]
    }
    /// 长度
    pub fn get_length(&self) -> f64 {
        (self.x().powi(2) + self.y().powi(2) + self.z().powi(2)).sqrt()
    }
    /// 单位向量
    pub fn get_unit(&self) -> Vector3 {
        *self / self.get_length()
    }
    /// 零长度向量
    pub fn zero() -> Vector3 {
        Vector3([0f64; 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut v1 = Vector3([1.0, 2.0, 3.0]);
        let v2 = Vector3([4.0, 5.0, 6.0]);

        let v1b = &mut v1;
        *v1b = v1b.add(v2);

        assert_eq!(v1, Vector3([5.0, 7.0, 9.0]));
    }
}
