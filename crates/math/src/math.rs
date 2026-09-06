use crate::{Matrix, ScalarOp, Vector};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

//
// Vectors

impl<T, const SIZE: usize> Add for Vector<T, SIZE>
where
    T: Add<Output = T> + Copy,
    Self: ScalarOp,
{
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            data: std::array::from_fn(|i| self[i] + rhs[i]),
        }
    }
}
impl<T, const SIZE: usize> Sub for Vector<T, SIZE>
where
    T: Sub<Output = T> + Copy,
    Self: ScalarOp,
{
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            data: std::array::from_fn(|i| self[i] - rhs[i]),
        }
    }
}
impl<T, const SIZE: usize> Mul for Vector<T, SIZE>
where
    T: Mul<Output = T> + Copy,
    Self: ScalarOp,
{
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            data: std::array::from_fn(|i| self[i] * rhs[i]),
        }
    }
}
impl<T, const SIZE: usize> Div for Vector<T, SIZE>
where
    T: Div<Output = T> + Copy,
    Self: ScalarOp,
{
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            data: std::array::from_fn(|i| self[i] / rhs[i]),
        }
    }
}

impl<T, const SIZE: usize> AddAssign for Vector<T, SIZE>
where
    T: AddAssign + Copy,
    Self: ScalarOp,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..SIZE {
            self[i] += rhs[i];
        }
    }
}
impl<T, const SIZE: usize> SubAssign for Vector<T, SIZE>
where
    T: SubAssign + Copy,
    Self: ScalarOp,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..SIZE {
            self[i] -= rhs[i];
        }
    }
}
impl<T, const SIZE: usize> MulAssign for Vector<T, SIZE>
where
    T: MulAssign + Copy,
    Self: ScalarOp,
{
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..SIZE {
            self[i] *= rhs[i];
        }
    }
}
impl<T, const SIZE: usize> DivAssign for Vector<T, SIZE>
where
    T: DivAssign + Copy,
    Self: ScalarOp,
{
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        for i in 0..SIZE {
            self[i] /= rhs[i];
        }
    }
}

//
// Matrices

impl<T, const M: usize, const N: usize> Add for Matrix<T, M, N>
where
    Self: ScalarOp,
    T: Add + Copy,
    Vector<T, N>: Add<Output = Vector<T, N>>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rows: Vector::<Vector<T, N>, M>::from(std::array::from_fn(|i| {
                self.rows[i] + rhs.rows[i]
            })),
        }
    }
}
impl<T, const M: usize, const N: usize> Sub for Matrix<T, M, N>
where
    Self: ScalarOp,
    T: Sub + Copy,
    Vector<T, N>: Sub<Output = Vector<T, N>>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            rows: Vector::<Vector<T, N>, M>::from(std::array::from_fn(|i| {
                self.rows[i] - rhs.rows[i]
            })),
        }
    }
}
impl<T, const M: usize, const N: usize, const RN: usize> Mul<Matrix<T, N, RN>> for Matrix<T, M, N>
where
    Self: ScalarOp,
    T: Default + Mul<Output = T> + Add<Output = T> + Copy,
{
    type Output = Matrix<T, M, RN>;

    fn mul(self, rhs: Matrix<T, N, RN>) -> Self::Output {
        Matrix::<T, M, RN>::from(std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                (0..N).fold(T::default(), |acc, k| acc + self[i][k] * rhs[k][j])
            })
        }))
    }
}
