use num_traits::{float::Float, Num};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};

/// 数的定义
pub trait Number: Copy + Num {} // 包含有意义浮点数和整数
impl Number for f64 {}
impl Number for f32 {}
impl Number for i64 {}
impl Number for i32 {}
impl Number for usize {}

/// 三维向量
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct Vec3<T: Number>(pub [T; 3]);

/// 类型转换
impl<T: Number> From<[T; 3]> for Vec3<T> {
    fn from(value: [T; 3]) -> Self {
        Vec3(value)
    }
}
/// 四则运算
impl<T: Number> Add<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T>;

    fn add(self, mut rhs: Vec3<T>) -> Self::Output {
        for i in 0..3 {
            rhs.0[i] = rhs.0[i] + self.0[i];
        }
        rhs
    }
}
impl<T: Number> Sub<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T>;

    fn sub(self, mut rhs: Vec3<T>) -> Self::Output {
        for i in 0..3 {
            rhs.0[i] = rhs.0[i] - self.0[i];
        }
        rhs
    }
}

// 对乘法不变的类型实现例如T:f64
// 如类型对乘法协变、逆变，需自己实现，如[i64;3] * f64 -> [f64;3]
impl<T: Number> Mul<T> for Vec3<T> {
    type Output = Vec3<T>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut rtn = [T::zero(); 3];
        for i in 0..3 {
            rtn[i] = self.0[i] * rhs;
        }
        rtn.into()
    }
}

// 对除法不变的类型实现
impl<T: Number> Div<T> for Vec3<T> {
    type Output = Vec3<T>;

    fn div(self, rhs: T) -> Self::Output {
        let mut rtn = [T::zero(); 3];
        for i in 0..3 {
            rtn[i] = self.0[i] / rhs;
        }
        Vec3::<T>(rtn)
    }
}

impl<T: Number> Vec3<T> {
    /// 三维
    pub fn x(&self) -> T {
        self.0[0]
    }
    pub fn y(&self) -> T {
        self.0[1]
    }
    pub fn z(&self) -> T {
        self.0[2]
    }
    pub fn zero() -> Vec3<T> {
        Vec3([T::zero(); 3])
    }
}

/// 从整数转换为实数
impl From<Vec3<i32>> for Vec3<f32> {
    fn from(value: Vec3<i32>) -> Self {
        let mut rtn = Vec3::<f32>([0.0; 3]);
        for i in 0..3 {
            rtn.0[i] = value.0[i] as f32;
        }
        rtn
    }
}
impl From<Vec3<i64>> for Vec3<f64> {
    fn from(value: Vec3<i64>) -> Self {
        let mut rtn = Vec3::<f64>([0.0; 3]);
        for i in 0..3 {
            rtn.0[i] = value.0[i] as f64;
        }
        rtn
    }
}
impl From<Vec3<usize>> for Vec3<f64> {
    fn from(value: Vec3<usize>) -> Self {
        let mut rtn = Vec3::<f64>([0.0; 3]);
        for i in 0..3 {
            rtn.0[i] = value.0[i] as f64;
        }
        rtn
    }
}
///对实数的额外功能
impl<T: Number + Float> Vec3<T> {
    /// 长度
    pub fn length(&self) -> T {
        (self.x().powi(2) + self.y().powi(2) + self.z().powi(2)).sqrt()
    }
    /// 单位向量
    pub fn unit(&self) -> Vec3<T> {
        *self / self.length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut v1 = Vec3::<f64>([1.0, 2.0, 3.0]);
        let v2 = Vec3::<f64>([4.0, 5.0, 6.0]);

        let v1b = &mut v1;
        *v1b = v1b.add(v2);

        assert_eq!(v1, Vec3([5.0, 7.0, 9.0]));
    }

    #[test]
    fn test_add_int() {
        let mut v1 = Vec3::<i64>([1, 2, 3]);
        let v2 = Vec3::<i64>([4, 5, 6]);

        let v1b = &mut v1;
        *v1b = v1b.add(v2);

        assert_eq!(v1, Vec3([5, 7, 9]));
    }

    #[test]
    fn test_length() {
        let v1 = Vec3::<i32>([3, 4, 0]);
        let v1_v: Vec3<f32> = v1.into();
        let v2 = Vec3::<f64>([4.0, 3.0, 0.0]);
        assert_eq!(v1_v.length(), 5.0);
        assert_eq!(v2.length(), 5.0);
    }
}
