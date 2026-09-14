use crate::{
    ScalarOp, Vector,
    util::{Identity, min},
};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct Matrix<T, const M: usize, const N: usize> {
    pub(crate) rows: Vector<Vector<T, N>, M>,
}

impl<T, const N: usize> Matrix<T, 2, N> {
    pub fn new(row0: Vector<T, N>, row1: Vector<T, N>) -> Self {
        Self {
            rows: Vector::<_, 2>::new(row0, row1),
        }
    }
}
impl<T, const N: usize> Matrix<T, 3, N> {
    pub fn new(row0: Vector<T, N>, row1: Vector<T, N>, row2: Vector<T, N>) -> Self {
        Self {
            rows: Vector::<_, 3>::new(row0, row1, row2),
        }
    }
}
impl<T, const N: usize> Matrix<T, 4, N> {
    pub fn new(
        row0: Vector<T, N>,
        row1: Vector<T, N>,
        row2: Vector<T, N>,
        row3: Vector<T, N>,
    ) -> Self {
        Self {
            rows: Vector::<_, 4>::new(row0, row1, row2, row3),
        }
    }
}

impl<T, const M: usize, const N: usize> Identity for Matrix<T, M, N>
where
    T: Default + Identity,
    Vector<Vector<T, N>, M>: Default,
{
    fn ident() -> Self {
        let mut rows = Vector::default();

        for i in 0..(min(M, N)) {
            rows[i][i] = T::ident();
        }

        Self { rows }
    }
}

impl<T, const M: usize, const N: usize> Deref for Matrix<T, M, N> {
    type Target = Vector<Vector<T, N>, M>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}
impl<T, const M: usize, const N: usize> DerefMut for Matrix<T, M, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rows
    }
}

impl<T, const M: usize, const N: usize> From<[[T; N]; M]> for Matrix<T, M, N>
where
    T: Copy,
{
    fn from(value: [[T; N]; M]) -> Self {
        Self {
            rows: Vector::<Vector<T, N>, M>::from(std::array::from_fn(|i| {
                Vector::<T, N>::from(value[i])
            })),
        }
    }
}

impl<T, const M: usize, const N: usize> Debug for Matrix<T, M, N>
where
    T: Debug + Copy + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;

        for i in 0..M {
            write!(f, "    [ ")?;
            for j in 0..N {
                if j < N - 1 {
                    write!(f, "{:?}, ", self[i][j])?;
                } else {
                    write!(f, "{:?}", self[i][j])?;
                }
            }
            writeln!(f, " ]")?;
        }

        writeln!(f, "]")
    }
}

impl<T, const M: usize, const N: usize> ScalarOp for Matrix<T, M, N> {}

pub type Mat2x2i8 = Matrix<i8, 2, 2>;
pub type Mat2x2i16 = Matrix<i16, 2, 2>;
pub type Mat2x2i32 = Matrix<i32, 2, 2>;
pub type Mat2x2i64 = Matrix<i64, 2, 2>;
pub type Mat2x2i128 = Matrix<i128, 2, 2>;
pub type Mat2x2isize = Matrix<isize, 2, 2>;
pub type Mat2x2u8 = Matrix<u8, 2, 2>;
pub type Mat2x2u16 = Matrix<u16, 2, 2>;
pub type Mat2x2u32 = Matrix<u32, 2, 2>;
pub type Mat2x2u64 = Matrix<u64, 2, 2>;
pub type Mat2x2u128 = Matrix<u128, 2, 2>;
pub type Mat2x2usize = Matrix<usize, 2, 2>;
pub type Mat2x2f32 = Matrix<f32, 2, 2>;
pub type Mat2x2f64 = Matrix<f64, 2, 2>;

pub type Mat2x3i8 = Matrix<i8, 2, 3>;
pub type Mat2x3i16 = Matrix<i16, 2, 3>;
pub type Mat2x3i32 = Matrix<i32, 2, 3>;
pub type Mat2x3i64 = Matrix<i64, 2, 3>;
pub type Mat2x3i128 = Matrix<i128, 2, 3>;
pub type Mat2x3isize = Matrix<isize, 2, 3>;
pub type Mat2x3u8 = Matrix<u8, 2, 3>;
pub type Mat2x3u16 = Matrix<u16, 2, 3>;
pub type Mat2x3u32 = Matrix<u32, 2, 3>;
pub type Mat2x3u64 = Matrix<u64, 2, 3>;
pub type Mat2x3u128 = Matrix<u128, 2, 3>;
pub type Mat2x3usize = Matrix<usize, 2, 3>;
pub type Mat2x3f32 = Matrix<f32, 2, 3>;
pub type Mat2x3f64 = Matrix<f64, 2, 3>;

pub type Mat2x4i8 = Matrix<i8, 2, 4>;
pub type Mat2x4i16 = Matrix<i16, 2, 4>;
pub type Mat2x4i32 = Matrix<i32, 2, 4>;
pub type Mat2x4i64 = Matrix<i64, 2, 4>;
pub type Mat2x4i128 = Matrix<i128, 2, 4>;
pub type Mat2x4isize = Matrix<isize, 2, 4>;
pub type Mat2x4u8 = Matrix<u8, 2, 4>;
pub type Mat2x4u16 = Matrix<u16, 2, 4>;
pub type Mat2x4u32 = Matrix<u32, 2, 4>;
pub type Mat2x4u64 = Matrix<u64, 2, 4>;
pub type Mat2x4u128 = Matrix<u128, 2, 4>;
pub type Mat2x4usize = Matrix<usize, 2, 4>;
pub type Mat2x4f32 = Matrix<f32, 2, 4>;
pub type Mat2x4f64 = Matrix<f64, 2, 4>;

pub type Mat3x2i8 = Matrix<i8, 3, 2>;
pub type Mat3x2i16 = Matrix<i16, 3, 2>;
pub type Mat3x2i32 = Matrix<i32, 3, 2>;
pub type Mat3x2i64 = Matrix<i64, 3, 2>;
pub type Mat3x2i128 = Matrix<i128, 3, 2>;
pub type Mat3x2isize = Matrix<isize, 3, 2>;
pub type Mat3x2u8 = Matrix<u8, 3, 2>;
pub type Mat3x2u16 = Matrix<u16, 3, 2>;
pub type Mat3x2u32 = Matrix<u32, 3, 2>;
pub type Mat3x2u64 = Matrix<u64, 3, 2>;
pub type Mat3x2u128 = Matrix<u128, 3, 2>;
pub type Mat3x2usize = Matrix<usize, 3, 2>;
pub type Mat3x2f32 = Matrix<f32, 3, 2>;
pub type Mat3x2f64 = Matrix<f64, 3, 2>;

pub type Mat3x3i8 = Matrix<i8, 3, 3>;
pub type Mat3x3i16 = Matrix<i16, 3, 3>;
pub type Mat3x3i32 = Matrix<i32, 3, 3>;
pub type Mat3x3i64 = Matrix<i64, 3, 3>;
pub type Mat3x3i128 = Matrix<i128, 3, 3>;
pub type Mat3x3isize = Matrix<isize, 3, 3>;
pub type Mat3x3u8 = Matrix<u8, 3, 3>;
pub type Mat3x3u16 = Matrix<u16, 3, 3>;
pub type Mat3x3u32 = Matrix<u32, 3, 3>;
pub type Mat3x3u64 = Matrix<u64, 3, 3>;
pub type Mat3x3u128 = Matrix<u128, 3, 3>;
pub type Mat3x3usize = Matrix<usize, 3, 3>;
pub type Mat3x3f32 = Matrix<f32, 3, 3>;
pub type Mat3x3f64 = Matrix<f64, 3, 3>;

pub type Mat3x4i8 = Matrix<i8, 3, 4>;
pub type Mat3x4i16 = Matrix<i16, 3, 4>;
pub type Mat3x4i32 = Matrix<i32, 3, 4>;
pub type Mat3x4i64 = Matrix<i64, 3, 4>;
pub type Mat3x4i128 = Matrix<i128, 3, 4>;
pub type Mat3x4isize = Matrix<isize, 3, 4>;
pub type Mat3x4u8 = Matrix<u8, 3, 4>;
pub type Mat3x4u16 = Matrix<u16, 3, 4>;
pub type Mat3x4u32 = Matrix<u32, 3, 4>;
pub type Mat3x4u64 = Matrix<u64, 3, 4>;
pub type Mat3x4u128 = Matrix<u128, 3, 4>;
pub type Mat3x4usize = Matrix<usize, 3, 4>;
pub type Mat3x4f32 = Matrix<f32, 3, 4>;
pub type Mat3x4f64 = Matrix<f64, 3, 4>;

pub type Mat4x2i8 = Matrix<i8, 4, 2>;
pub type Mat4x2i16 = Matrix<i16, 4, 2>;
pub type Mat4x2i32 = Matrix<i32, 4, 2>;
pub type Mat4x2i64 = Matrix<i64, 4, 2>;
pub type Mat4x2i128 = Matrix<i128, 4, 2>;
pub type Mat4x2isize = Matrix<isize, 4, 2>;
pub type Mat4x2u8 = Matrix<u8, 4, 2>;
pub type Mat4x2u16 = Matrix<u16, 4, 2>;
pub type Mat4x2u32 = Matrix<u32, 4, 2>;
pub type Mat4x2u64 = Matrix<u64, 4, 2>;
pub type Mat4x2u128 = Matrix<u128, 4, 2>;
pub type Mat4x2usize = Matrix<usize, 4, 2>;
pub type Mat4x2f32 = Matrix<f32, 4, 2>;
pub type Mat4x2f64 = Matrix<f64, 4, 2>;

pub type Mat4x3i8 = Matrix<i8, 4, 3>;
pub type Mat4x3i16 = Matrix<i16, 4, 3>;
pub type Mat4x3i32 = Matrix<i32, 4, 3>;
pub type Mat4x3i64 = Matrix<i64, 4, 3>;
pub type Mat4x3i128 = Matrix<i128, 4, 3>;
pub type Mat4x3isize = Matrix<isize, 4, 3>;
pub type Mat4x3u8 = Matrix<u8, 4, 3>;
pub type Mat4x3u16 = Matrix<u16, 4, 3>;
pub type Mat4x3u32 = Matrix<u32, 4, 3>;
pub type Mat4x3u64 = Matrix<u64, 4, 3>;
pub type Mat4x3u128 = Matrix<u128, 4, 3>;
pub type Mat4x3usize = Matrix<usize, 4, 3>;
pub type Mat4x3f32 = Matrix<f32, 4, 3>;
pub type Mat4x3f64 = Matrix<f64, 4, 3>;

pub type Mat4x4i8 = Matrix<i8, 4, 4>;
pub type Mat4x4i16 = Matrix<i16, 4, 4>;
pub type Mat4x4i32 = Matrix<i32, 4, 4>;
pub type Mat4x4i64 = Matrix<i64, 4, 4>;
pub type Mat4x4i128 = Matrix<i128, 4, 4>;
pub type Mat4x4isize = Matrix<isize, 4, 4>;
pub type Mat4x4u8 = Matrix<u8, 4, 4>;
pub type Mat4x4u16 = Matrix<u16, 4, 4>;
pub type Mat4x4u32 = Matrix<u32, 4, 4>;
pub type Mat4x4u64 = Matrix<u64, 4, 4>;
pub type Mat4x4u128 = Matrix<u128, 4, 4>;
pub type Mat4x4usize = Matrix<usize, 4, 4>;
pub type Mat4x4f32 = Matrix<f32, 4, 4>;
pub type Mat4x4f64 = Matrix<f64, 4, 4>;
