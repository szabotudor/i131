use crate::traits::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vector2Base<T>
where
    _VectorBaseHelper<T>: IsVectorBase<2>,
    T: Clone + Copy,
{
    pub x: T,
    pub y: T,
}

impl<T> IsVectorBase<2> for _VectorBaseHelper<T>
where
    T: Clone + Copy,
{
    type Vector = Vector2Base<T>;
}

impl<T> Default for Vector2Base<T>
where
    T: Default + Clone + Copy,
    _VectorBaseHelper<T>: IsVectorBase<2>,
{
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<'a, T> VectorBaseStorage<T, 2> for Vector2Base<T>
where
    T: Clone + Copy,
{
    fn read<const N: usize>(&self) -> T {
        match N {
            0 => self.x,
            1 => self.y,
            _ => {
                unreachable!("Wrong vector accessor")
            }
        }
    }

    fn write<const N: usize>(&mut self, v: T) {
        match N {
            0 => self.x = v,
            1 => self.y = v,
            _ => {
                unreachable!("Wrong vector accessor")
            }
        }
    }
}
