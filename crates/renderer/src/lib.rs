use math131::{Vec2u32, Vec4f32};
use serde::{Deserialize, Serialize};
use std::{
    any::TypeId,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};
use thiserror::Error;
use utils131::define_handle;

pub mod build_tools;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Failed to initialize renderer: {0}")]
    InitError(String),

    #[error("Error in renderer API usage: {0}")]
    APIError(String),

    #[error(
        "Vertex buffer info declared stride as {0}, but calculated stride was {1}. Fields are: {2:?}"
    )]
    VertexBufferStrideMismatch(usize, usize, Vec<BufferItemFieldInfo>),

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

define_handle! {ExecutionContext}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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

define_handle! {Shader}

define_handle! {Program}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    Float,
    SInt,
    UInt,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentBitCount {
    Scalar(u8),
    Two { a: u8, b: u8 },
    Three { r: u8, g: u8, b: u8 },
    Four { r: u8, g: u8, b: u8, a: u8 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferFieldFormat {
    pub kind: ScalarKind,
    pub normalized: bool,
    pub bits_per_component: ComponentBitCount,
}
#[derive(Debug, Clone)]
pub struct BufferItemFieldInfo {
    pub size: usize,
    pub offset_in_item: usize,
    pub format: BufferFieldFormat,
}
#[derive(Hash, PartialEq, Eq)]
pub enum BufferBinding {
    Name(String),
    Location(usize),
    IndexBufferBinding,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    Vertex,
    Index,
}
pub struct BufferCreateInfo<'a> {
    pub usage: BufferUsage,
    pub data: &'a [u8],
    pub item_stride: usize,
    pub item_count: usize,
    pub item_fields: HashMap<BufferBinding, BufferItemFieldInfo>,
}
impl<'a> BufferCreateInfo<'a> {
    pub fn new<T: 'static>(usage: BufferUsage, raw_data: &'a [T]) -> Self {
        let res = Self {
            usage,
            data: unsafe {
                std::slice::from_raw_parts(
                    raw_data.as_ptr() as *const u8,
                    std::mem::size_of_val(raw_data),
                )
            },
            item_stride: size_of::<T>(),
            item_count: raw_data.len(),
            item_fields: HashMap::default(),
        };

        if usage == BufferUsage::Index {
            if TypeId::of::<T>() != TypeId::of::<u32>() {
                panic!("Index buffer type must be u32");
            }

            res.with_field::<u32>(
                BufferBinding::IndexBufferBinding,
                0,
                BufferFieldFormat {
                    kind: ScalarKind::UInt,
                    normalized: false,
                    bits_per_component: ComponentBitCount::Scalar(32),
                },
            )
        } else {
            res
        }
    }

    pub fn with_field<T: 'static>(
        mut self,
        shader_buffer_binding: BufferBinding,
        offset_in_item: usize,
        format: BufferFieldFormat,
    ) -> Self {
        if self.usage == BufferUsage::Index {
            if !self.item_fields.is_empty() {
                panic!(
                    "Index buffer should contain exactly one field (the index).\nDefault field is automatically generated when creating an index buffer"
                );
            }
            if format
                != (BufferFieldFormat {
                    kind: ScalarKind::UInt,
                    normalized: false,
                    bits_per_component: ComponentBitCount::Scalar(32),
                })
            {
                panic!("Unsupported format for index buffer: {format:?}");
            }
        }

        self.item_fields.insert(
            shader_buffer_binding,
            BufferItemFieldInfo {
                size: size_of::<T>(),
                offset_in_item,
                format,
            },
        );
        self
    }

    pub fn validate(&self) -> Result<(), RendererError> {
        let expected_item_stride = self
            .item_fields
            .iter()
            .fold(0usize, |acc, (_, field)| acc + field.size);

        if expected_item_stride == self.item_stride {
            Ok(())
        } else {
            Err(RendererError::VertexBufferStrideMismatch(
                self.item_stride,
                expected_item_stride,
                self.item_fields.values().cloned().collect(),
            ))
        }
    }
}
define_handle! {Buffer}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Viewport {
    pub origin: Vec2u32,
    pub size: Vec2u32,
}
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Settings {
    pub clear_color: Vec4f32,
    /// Also used for raw framebuffer size when needed
    pub viewport: Viewport,
}

#[derive(Clone, Debug)]
pub enum DrawMode {
    Raw { vertex_count: u64 },
    Indexed,
}
#[derive(Clone)]
pub enum DrawCall {
    Draw {
        program: ProgramHandle,
        buffers: Vec<BufferHandle>,
        mode: DrawMode,
    },
}

pub trait Renderer
where
    Self: Send + Sync,
{
    fn name(&self) -> &'static str;
    fn max_execution_contexts(&self) -> usize;

    fn create_execution_context(&mut self) -> Result<ExecutionContextHandle, RendererError>;
    fn destroy_execution_context(
        &mut self,
        context: ExecutionContextHandle,
    ) -> Result<(), RendererError>;

    fn destroy(&mut self) -> Result<(), RendererError>;

    fn create_shader(&mut self, info: ShaderCreateInfo) -> Result<ShaderHandle, RendererError>;
    fn create_shaders(
        &mut self,
        infos: &[ShaderCreateInfo],
    ) -> Result<Vec<ShaderHandle>, RendererError>;
    fn destroy_shader(&mut self, shader: ShaderHandle) -> Result<(), RendererError>;
    fn destroy_shaders(&mut self, shaders: &[ShaderHandle]) -> Result<(), RendererError>;

    fn create_program(&mut self, shaders: &[ShaderHandle]) -> Result<ProgramHandle, RendererError>;
    fn destroy_program(&mut self, program: ProgramHandle) -> Result<(), RendererError>;

    fn create_buffer(&mut self, data: BufferCreateInfo) -> Result<BufferHandle, RendererError>;
    fn destroy_buffer(&mut self, vertex_buffer: BufferHandle) -> Result<(), RendererError>;

    fn execute(
        &mut self,
        draw_call: DrawCall,
        context: ExecutionContextHandle,
    ) -> Result<(), RendererError>;

    fn change_settings(
        &mut self,
        callback: Box<dyn FnMut(&mut Settings)>,
    ) -> Result<(), RendererError>;
}
