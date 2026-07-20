use std::fmt::{Debug, Display};
use thiserror::Error;
pub use window131::Window;

pub mod build_tools;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Failed to initialize renderer: {0}")]
    InitError(String),

    #[error("Error in renderer: {0}")]
    InstanceError(Box<dyn RendererInstanceError>),
}
unsafe impl Send for RendererError {}
pub trait RendererInstanceError
where
    Self: Debug + Display,
{
}
impl<T> From<T> for RendererError
where
    T: RendererInstanceError + 'static,
{
    fn from(value: T) -> Self {
        RendererError::InstanceError(Box::new(value))
    }
}

pub trait OptionRendererError<T> {
    fn ok_or_renderer_error(self, err: RendererError) -> Result<T, RendererError>;
}
impl<T> OptionRendererError<T> for Option<T> {
    fn ok_or_renderer_error(self, err: RendererError) -> Result<T, RendererError> {
        if let Some(opt) = self {
            Ok(opt)
        } else {
            Err(err)
        }
    }
}

pub struct ShaderID {
    pub raw: usize,
}

pub trait Renderer
where
    Self: Send + Sync,
{
    fn name(&self) -> &'static str;

    fn create_shader(&mut self, source: &[u8]) -> Result<usize, RendererError>;
}
