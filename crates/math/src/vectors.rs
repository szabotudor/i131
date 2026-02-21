use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::traits::*;
use crate::base_vectors::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector<T, const SIZE: usize>
where
    VectorBaseType<T>: VectorBaseData<SIZE>,
    <VectorBaseType<T> as VectorBaseData<SIZE>>::Vector: VectorBaseStorage<T, SIZE> + Clone + Copy,
{
    _v: <VectorBaseType<T> as VectorBaseData<SIZE>>::Vector,
}

impl<'a, T, const SIZE: usize> Default for Vector<T, SIZE>
where
    VectorBaseType<T>: VectorBaseData<SIZE>,
    T: Default,
    <VectorBaseType<T> as VectorBaseData<SIZE>>::Vector:
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
    VectorBaseType<T>: VectorBaseData<SIZE>,
    <VectorBaseType<T> as VectorBaseData<SIZE>>::Vector: VectorBaseStorage<T, SIZE> + Clone + Copy,
{
    type Target = <VectorBaseType<T> as VectorBaseData<SIZE>>::Vector;

    fn deref(&self) -> &Self::Target {
        &self._v
    }
}
impl<'a, T, const SIZE: usize> DerefMut for Vector<T, SIZE>
where
    VectorBaseType<T>: VectorBaseData<SIZE>,
    <VectorBaseType<T> as VectorBaseData<SIZE>>::Vector: VectorBaseStorage<T, SIZE> + Clone + Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._v
    }
}

impl<'a, T, const SIZE: usize> Debug for Vector<T, SIZE>
where
    VectorBaseType<T>: VectorBaseData<SIZE>,
    <VectorBaseType<T> as VectorBaseData<SIZE>>::Vector:
        VectorBaseStorage<T, SIZE> + Clone + Copy + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._v.fmt(f)
    }
}

pub type Vec2f32<'a> = Vector<f32, 2>;
pub type Vec2f64<'a> = Vector<f64, 2>;
pub type Vec2u8 = Vector2Base<u8>;
pub type Vec2i8 = Vector2Base<i8>;
pub type Vec2u16 = Vector2Base<u16>;
pub type Vec2i16 = Vector2Base<i16>;
pub type Vec2u32 = Vector2Base<u32>;
pub type Vec2i32 = Vector2Base<i32>;
pub type Vec2u64 = Vector2Base<u64>;
pub type Vec2i64 = Vector2Base<i64>;
