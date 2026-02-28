use std::{
    array::IntoIter,
    fmt::Debug,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::{Iter, IterMut},
};

//===============
// Vector struct
//===============

#[repr(C)]
pub struct Vector<T, const SIZE: usize> {
    pub(crate) data: [T; SIZE],
}

impl<T> Vector<T, 2> {
    pub fn new(x: T, y: T) -> Self {
        Self { data: [x, y] }
    }
}
impl<T> Vector<T, 3> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { data: [x, y, z] }
    }
}
impl<T> Vector<T, 4> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { data: [x, y, z, w] }
    }
}

impl<T, const SIZE: usize> Default for Vector<T, SIZE>
where
    [T; SIZE]: Default,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<T, const SIZE: usize> From<T> for Vector<T, SIZE>
where
    T: Copy,
{
    fn from(value: T) -> Self {
        Self {
            data: std::array::from_fn(|_| value),
        }
    }
}

impl<T, const SIZE: usize> Debug for Vector<T, SIZE>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match SIZE {
            2 => f
                .debug_struct("Vector2")
                .field("x", &self.data[0])
                .field("y", &self.data[1])
                .finish(),
            3 => f
                .debug_struct("Vector3")
                .field("x", &self.data[0])
                .field("y", &self.data[1])
                .field("z", &self.data[2])
                .finish(),
            4 => f
                .debug_struct("Vector4")
                .field("x", &self.data[0])
                .field("y", &self.data[1])
                .field("z", &self.data[2])
                .field("w", &self.data[3])
                .finish(),
            _ => f.debug_tuple("Vector").field(&self.data).finish(),
        }
    }
}

impl<T, const SIZE: usize> Clone for Vector<T, SIZE>
where
    [T; SIZE]: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}
impl<T, const SIZE: usize> Copy for Vector<T, SIZE>
where
    [T; SIZE]: Copy,
    T: Copy,
{
}

impl<'a, T, const SIZE: usize> IntoIterator for &'a Vector<T, SIZE> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
impl<'a, T, const SIZE: usize> IntoIterator for &'a mut Vector<T, SIZE> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}
impl<T, const SIZE: usize> IntoIterator for Vector<T, SIZE> {
    type Item = T;
    type IntoIter = IntoIter<T, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.data)
    }
}

impl<T, const SIZE: usize> Index<usize> for Vector<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<T, const SIZE: usize> IndexMut<usize> for Vector<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

//=============================================
// Storages (beautiful access into the vector)
//=============================================

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector2Storage<T> {
    pub x: T,
    pub y: T,
}
impl<T> Deref for Vector<T, 2> {
    type Target = Vector2Storage<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<T> DerefMut for Vector<T, 2> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3Storage<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}
impl<T> Deref for Vector<T, 3> {
    type Target = Vector3Storage<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<T> DerefMut for Vector<T, 3> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector4Storage<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}
impl<T> Deref for Vector<T, 4> {
    type Target = Vector4Storage<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<T> DerefMut for Vector<T, 4> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

//================
// Common Vectors
//================

pub trait ScalarOp {}
#[cfg(feature = "SIMD")]
pub trait SIMDOp {}

#[cfg(not(feature = "SIMD"))]
impl<T, const SIZE: usize> ScalarOp for Vector<T, SIZE> {}

#[cfg(feature = "SIMD")]
mod simd {
    use crate::*;

    // SIMD vectors
    impl SIMDOp for Vec4f32 {}

    // Non-SIMD (scalar) vectors
    impl ScalarOp for Vec2f32 {}
    impl ScalarOp for Vec3f32 {}

    impl ScalarOp for Vec2f64 {}
    impl ScalarOp for Vec3f64 {}
    impl ScalarOp for Vec4f64 {}

    impl ScalarOp for Vec2u8 {}
    impl ScalarOp for Vec2i8 {}
    impl ScalarOp for Vec2u16 {}
    impl ScalarOp for Vec2i16 {}
    impl ScalarOp for Vec2u32 {}
    impl ScalarOp for Vec2i32 {}
    impl ScalarOp for Vec2u64 {}
    impl ScalarOp for Vec2i64 {}

    impl ScalarOp for Vec3u8 {}
    impl ScalarOp for Vec3i8 {}
    impl ScalarOp for Vec3u16 {}
    impl ScalarOp for Vec3i16 {}
    impl ScalarOp for Vec3u32 {}
    impl ScalarOp for Vec3i32 {}
    impl ScalarOp for Vec3u64 {}
    impl ScalarOp for Vec3i64 {}

    impl ScalarOp for Vec4u8 {}
    impl ScalarOp for Vec4i8 {}
    impl ScalarOp for Vec4u16 {}
    impl ScalarOp for Vec4i16 {}
    impl ScalarOp for Vec4u32 {}
    impl ScalarOp for Vec4i32 {}
    impl ScalarOp for Vec4u64 {}
    impl ScalarOp for Vec4i64 {}
}

pub type Vec2f32 = Vector<f32, 2>;
pub type Vec2f64 = Vector<f64, 2>;

pub type Vec2u8 = Vector<u8, 2>;
pub type Vec2i8 = Vector<i8, 2>;
pub type Vec2u16 = Vector<u16, 2>;
pub type Vec2i16 = Vector<i16, 2>;
pub type Vec2u32 = Vector<u32, 2>;
pub type Vec2i32 = Vector<i32, 2>;
pub type Vec2u64 = Vector<u64, 2>;
pub type Vec2i64 = Vector<i64, 2>;

pub type Vec3f32 = Vector<f32, 3>;
pub type Vec3f64 = Vector<f64, 3>;

pub type Vec3u8 = Vector<u8, 3>;
pub type Vec3i8 = Vector<i8, 3>;
pub type Vec3u16 = Vector<u16, 3>;
pub type Vec3i16 = Vector<i16, 3>;
pub type Vec3u32 = Vector<u32, 3>;
pub type Vec3i32 = Vector<i32, 3>;
pub type Vec3u64 = Vector<u64, 3>;
pub type Vec3i64 = Vector<i64, 3>;

pub type Vec4f32 = Vector<f32, 4>;
pub type Vec4f64 = Vector<f64, 4>;

pub type Vec4u8 = Vector<u8, 4>;
pub type Vec4i8 = Vector<i8, 4>;
pub type Vec4u16 = Vector<u16, 4>;
pub type Vec4i16 = Vector<i16, 4>;
pub type Vec4u32 = Vector<u32, 4>;
pub type Vec4i32 = Vector<i32, 4>;
pub type Vec4u64 = Vector<u64, 4>;
pub type Vec4i64 = Vector<i64, 4>;
