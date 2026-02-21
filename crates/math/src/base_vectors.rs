use crate::traits::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vector2Base<T>
where
    VectorBaseType<T>: VectorBaseData<2>,
{
    pub x: T,
    pub y: T,
}

impl<T> VectorBaseData<2> for VectorBaseType<T> {
    type Vector = Vector2Base<T>;
}

impl<T> Default for Vector2Base<T>
where
    T: Default,
    VectorBaseType<T>: VectorBaseData<2>,
{
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<'a, T> VectorBaseStorage<T, 2> for Vector2Base<T> {
    fn data(&self) -> &[T; 2] {
        todo!()
    }

    fn data_mut(&mut self) -> &mut [T; 2] {
        todo!()
    }
}
