use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::traits::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector<T, const SIZE: usize>
where
    _VectorBaseHelper<T>: IsVectorBase<SIZE>,
    <_VectorBaseHelper<T> as IsVectorBase<SIZE>>::Vector: VectorBaseStorage<T, SIZE> + Clone + Copy,
{
    _v: <_VectorBaseHelper<T> as IsVectorBase<SIZE>>::Vector,
}

impl<'a, T, const SIZE: usize> Default for Vector<T, SIZE>
where
    _VectorBaseHelper<T>: IsVectorBase<SIZE>,
    T: Default,
    <_VectorBaseHelper<T> as IsVectorBase<SIZE>>::Vector:
        Default + VectorBaseStorage<T, SIZE> + Clone + Copy,
{
    fn default() -> Self {
        Self {
            _v: Default::default(),
        }
    }
}

impl<'a, T, const SIZE: usize> Deref for Vector<T, SIZE>
where
    _VectorBaseHelper<T>: IsVectorBase<SIZE>,
    <_VectorBaseHelper<T> as IsVectorBase<SIZE>>::Vector: VectorBaseStorage<T, SIZE> + Clone + Copy,
{
    type Target = <_VectorBaseHelper<T> as IsVectorBase<SIZE>>::Vector;

    fn deref(&self) -> &Self::Target {
        &self._v
    }
}
impl<'a, T, const SIZE: usize> DerefMut for Vector<T, SIZE>
where
    _VectorBaseHelper<T>: IsVectorBase<SIZE>,
    <_VectorBaseHelper<T> as IsVectorBase<SIZE>>::Vector: VectorBaseStorage<T, SIZE> + Clone + Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._v
    }
}

impl<'a, T, const SIZE: usize> Debug for Vector<T, SIZE>
where
    _VectorBaseHelper<T>: IsVectorBase<SIZE>,
    <_VectorBaseHelper<T> as IsVectorBase<SIZE>>::Vector:
        VectorBaseStorage<T, SIZE> + Clone + Copy + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._v.fmt(f)
    }
}

macro_rules! vec_shortcut {
    {$name:ident, $base:ty, $size:expr} => {
        #[repr(transparent)]
        pub struct $name { _v: Vector<$base, $size> }
        impl Deref for $name {
            type Target = Vector<$base, $size>;
            fn deref(&self) -> &Self::Target {
                &self._v
            }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self._v
            }
        }
        impl Default for $name where Vector<$base, $size>: Default {
            fn default() -> Self {
                Self { _v: Default::default() }
            }
        }
    };
}

vec_shortcut! {Vec2f32, f32, 2}
vec_shortcut! {Vec2f64, f64, 2}
vec_shortcut! {Vec2u8, u8, 2}
vec_shortcut! {Vec2i8, i8, 2}
vec_shortcut! {Vec2u16, u16, 2}
vec_shortcut! {Vec2i16, i16, 2}
vec_shortcut! {Vec2u32, u32, 2}
vec_shortcut! {Vec2i32, i32, 2}
vec_shortcut! {Vec2u64, u64, 2}
vec_shortcut! {Vec2i64, i64, 2}
