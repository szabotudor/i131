pub struct _VectorBaseHelper<T> {
    _t: T,
}
pub trait IsVectorBase<const SIZE: usize> {
    type Vector;
}

pub trait VectorBaseStorage<T, const SIZE: usize>
where
    Self: Sized,
{
    fn read<const N: usize>(&self) -> T;
    fn write<const N: usize>(&mut self, v: T);
}
