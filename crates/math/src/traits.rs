pub struct VectorBaseType<T> {
    _t: T,
}
pub trait VectorBaseData<const SIZE: usize> {
    type Vector;
}

pub trait VectorBaseStorage<T, const SIZE: usize>
where
    Self: Sized,
{
    fn data(&self) -> &[T; SIZE];
    fn data_mut(&mut self) -> &mut [T; SIZE];
}
