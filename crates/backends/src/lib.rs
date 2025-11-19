use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct BackendInfo {
    pub name: String,
}

pub trait Backend
where
    Self: std::fmt::Debug,
{
    fn info(self) -> BackendInfo;

    fn init(&mut self) -> Result<()>;
}
