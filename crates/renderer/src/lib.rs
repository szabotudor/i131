use thiserror::Error;
pub use window131::Window;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Failed to initialize renderer: {0}")]
    InitError(String),
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

pub trait Renderer
where
    Self: Send + Sync,
{
}
