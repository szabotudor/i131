use math131::Vec4f32;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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

pub struct Handle<Marker> {
    raw: usize,
    _marker: PhantomData<Marker>,
}
trait HandleName {
    const NAME: &'static str;
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Handle<T> {}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<T> Eq for Handle<T> {}

impl<T> Handle<T> {
    pub fn null() -> Self {
        Self {
            raw: usize::MAX,
            _marker: Default::default(),
        }
    }

    pub fn is_null(&self) -> bool {
        self.eq(&Self::null())
    }

    pub fn from_raw(raw: usize) -> Self {
        Self {
            raw,
            _marker: Default::default(),
        }
    }
    pub fn as_raw(&self) -> usize {
        self.raw
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}
impl<T> Debug for Handle<T>
where
    T: HandleName,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle<{}>({:#x})", T::NAME, self.raw,)
    }
}

enum Slot<Data> {
    Occupied(Data),
    Free { next_free_idx: Option<usize> },
}
#[derive(Default)]
pub struct HandleMap<Handle, Data> {
    slots: Vec<Slot<Data>>,
    first_free_idx: Option<usize>,
    _marker: PhantomData<Handle>,
}
impl<Marker, Data> Default for HandleMap<Handle<Marker>, Data> {
    fn default() -> Self {
        Self {
            slots: Default::default(),
            first_free_idx: Default::default(),
            _marker: Default::default(),
        }
    }
}
impl<Marker, Data> HandleMap<Handle<Marker>, Data> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, data: Data) -> Handle<Marker> {
        if let Some(idx) = self.first_free_idx {
            if let Slot::Free { next_free_idx } = self.slots[idx] {
                self.first_free_idx = next_free_idx;
                self.slots[idx] = Slot::Occupied(data);
                Handle::from_raw(idx)
            } else {
                unreachable!("Data corruption: Expected Slot::Free");
            }
        } else {
            let idx = self.slots.len();
            self.slots.push(Slot::Occupied(data));
            Handle::from_raw(idx)
        }
    }

    pub fn remove(&mut self, handle: Handle<Marker>) -> Option<Data> {
        let idx = handle.as_raw();
        if idx >= self.slots.len() {
            return None;
        }

        if let Slot::Occupied(_) = self.slots[idx] {
            if let Slot::Occupied(data) = std::mem::replace(
                &mut self.slots[idx],
                Slot::Free {
                    next_free_idx: self.first_free_idx,
                },
            ) {
                self.first_free_idx = Some(idx);
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.first_free_idx = None;
    }

    pub fn get(&self, handle: Handle<Marker>) -> Option<&Data> {
        match self.slots.get(handle.as_raw()) {
            Some(Slot::Occupied(data)) => Some(data),
            _ => None,
        }
    }
    pub fn get_mut(&mut self, handle: Handle<Marker>) -> Option<&mut Data> {
        match self.slots.get_mut(handle.as_raw()) {
            Some(Slot::Occupied(data)) => Some(data),
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Handle<Marker>, &Data)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| match slot {
                Slot::Occupied(data) => Some((Handle::<Marker>::from_raw(idx), data)),
                Slot::Free { .. } => None,
            })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle<Marker>, &mut Data)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, slot)| match slot {
                Slot::Occupied(data) => Some((Handle::<Marker>::from_raw(idx), data)),
                Slot::Free { .. } => None,
            })
    }
}

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
pub struct ShaderHandleMarker;
impl HandleName for ShaderHandleMarker {
    const NAME: &'static str = "Shader";
}
pub type ShaderHandle = Handle<ShaderHandleMarker>;

pub struct ProgramHandleMarker;
impl HandleName for ProgramHandleMarker {
    const NAME: &'static str = "ShaderProgram";
}
pub type ProgramHandle = Handle<ProgramHandleMarker>;

#[derive(Default, Clone, Debug)]
pub struct Settings {
    pub clear_color: Vec4f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ScalarKind {
    Float,
    SInt,
    UInt,
}
#[derive(Debug, Clone, Copy)]
pub enum ComponentBitCount {
    Scalar(u8),
    Two { a: u8, b: u8 },
    Three { r: u8, g: u8, b: u8 },
    Four { r: u8, g: u8, b: u8, a: u8 },
}
#[derive(Debug, Clone, Copy)]
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
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    Vertex,
}
pub struct BufferCreateInfo<'a> {
    pub usage: BufferUsage,
    pub data: &'a [u8],
    pub item_stride: usize,
    pub item_count: usize,
    pub item_fields: HashMap<BufferBinding, BufferItemFieldInfo>,
}
impl<'a> BufferCreateInfo<'a> {
    pub fn new<T>(usage: BufferUsage, raw_data: &'a [T]) -> Self {
        Self {
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
        }
    }

    pub fn with_field<T: 'static>(
        mut self,
        shader_buffer_binding: BufferBinding,
        offset_in_item: usize,
        format: BufferFieldFormat,
    ) -> Self {
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
pub struct BufferHandleMarker;
impl HandleName for BufferHandleMarker {
    const NAME: &'static str = "VertexBuffer";
}
pub type BufferHandle = Handle<BufferHandleMarker>;

#[derive(Clone)]
pub enum DrawCall {
    Draw {
        program: ProgramHandle,
        vertex_buffers: Vec<BufferHandle>,
    },
}

pub trait Renderer
where
    Self: Send + Sync,
{
    fn name(&self) -> &'static str;

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

    fn execute(&mut self, draw_call: DrawCall) -> Result<(), RendererError>;
}
