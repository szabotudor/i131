use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

//===============
// Vector struct
//===============

#[repr(C)]
pub struct Vector<T, const SIZE: usize> {
    pub(crate) data: [T; SIZE],
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
        unsafe { &*(&self.data as *const T as *const Self::Target) }
    }
}
impl<T> DerefMut for Vector<T, 2> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut self.data as *mut T as *mut Self::Target) }
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
        unsafe { &*(&self.data as *const T as *const Self::Target) }
    }
}
impl<T> DerefMut for Vector<T, 3> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut self.data as *mut T as *mut Self::Target) }
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
        unsafe { &*(&self.data as *const T as *const Self::Target) }
    }
}
impl<T> DerefMut for Vector<T, 4> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut self.data as *mut T as *mut Self::Target) }
    }
}

//================
// Common Vectors
//================

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
