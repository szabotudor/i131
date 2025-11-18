#[derive(Debug)]
pub struct BackendInfo {
    pub name: String,
}

pub trait Backend
where
    Self: Sized,
{
    fn info() -> BackendInfo;
}
