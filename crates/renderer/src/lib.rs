use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};
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

pub struct Handle<Marker> {
    raw: u32,
    generation: u32,
    _marker: PhantomData<Marker>,
}
trait HandleName {
    const NAME: &'static str;
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            generation: self.generation.clone(),
            _marker: Default::default(),
        }
    }
}
impl<T> Copy for Handle<T> {}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw && self.generation == other.generation
    }
}
impl<T> Eq for Handle<T> {}

impl<T> Handle<T> {
    pub fn null() -> Self {
        Self {
            raw: std::u32::MAX,
            generation: std::u32::MAX,
            _marker: Default::default(),
        }
    }

    pub fn is_null(&self) -> bool {
        self.eq(&Self::null())
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
        self.generation.hash(state);
    }
}
impl<T> Debug for Handle<T>
where
    T: HandleName,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Handle<{}>({:#x}:{:#x})",
            T::NAME,
            self.raw,
            self.generation
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ShaderStage {
    Vertex,
    Pixel,
    Compute,
}
pub struct ShaderCreateInfo<'a> {
    pub source: &'a [u8],
    pub stage: ShaderStage,
    pub name: String,
}
pub struct ShaderHandleMarker;
impl HandleName for ShaderHandleMarker {
    const NAME: &'static str = "Shader";
}
pub type ShaderHandle = Handle<ShaderHandleMarker>;

pub trait Renderer
where
    Self: Send + Sync,
{
    fn name(&self) -> &'static str;

    fn create_shader(&mut self, info: ShaderCreateInfo) -> Result<ShaderHandle, RendererError>;
    fn destroy_shader(&mut self, shader: ShaderHandle) -> Result<(), RendererError>;
}
