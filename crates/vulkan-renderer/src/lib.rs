use crate::vulkan_init::SwapchainSupportDetails;
use ash::{Device, Entry, Instance, LoadingError, vk};
use math131::Vec2i32;
#[cfg(feature = "GLFW")]
use raw_window_handle::HandleError;
#[cfg(feature = "GLFW")]
use renderer131::RendererError;
use renderer131::{
    BufferCreateInfo, BufferFieldFormat, BufferHandle, BufferUsage, DrawCall, HandleMap,
    ProgramHandle, Renderer, RendererInstanceError, Settings, ShaderCreateInfo, ShaderHandle,
    ShaderStage,
};
#[cfg(feature = "GLFW")]
use std::{cell::RefCell, rc::Rc};
use std::{
    collections::{HashMap, VecDeque},
    ffi::{CString, NulError, c_void},
    sync::{Arc, RwLock},
};
use thiserror::Error;
#[cfg(feature = "GLFW")]
use window131::{Window, WindowDataGLFW};

pub mod build_tools;
mod vulkan_impl;
mod vulkan_init;

#[derive(Error, Debug)]
pub enum VulkanRendererError {
    #[cfg(feature = "GLFW")]
    #[error("Error getting GLFW instance")]
    GLFWInstanceError,

    #[error("Window RefCell should not be borrowed when `execute` is invoked on a vulkan renderer")]
    WindowAlreadyBorrowedError,

    #[cfg(feature = "GLFW")]
    #[error("Unknown GLFW error: {0}")]
    UnknownGLFWError(String),

    #[cfg(feature = "GLFW")]
    #[error("Error getting raw window handle for vkSurfaceKHR creation: {0}")]
    HandleError(HandleError),

    #[error("Cannot enable validation layers because they are not supported")]
    ValidationLayersNotSupported,

    #[error("Failed to create debug messenger")]
    InvalidDebugMessenger,

    #[error("Failed to create {0} vulkan surface")]
    SurfaceCreateFailure(String),

    #[error("Missing required vulkan extension \"{0}\"")]
    MissingExtention(String),

    #[error("Chosen devise is missing required queue family support for \"{0}\"")]
    MissingQueue(String),

    #[error("Shader handle {0:?} doesn't exist")]
    NonexistantShader(ShaderHandle),

    #[error("Buffer handle {0:?} doesn't exist")]
    NonexistantBuffer(BufferHandle),

    #[error("Could not find a supported memory layout for buffer")]
    NoSupportedMemoryLayouts,

    #[error("There are no physical devices that support vulkan")]
    NoSupportedDevices,

    #[error("Requested vertex format isn't supported by Vulkan backend: {0:?}")]
    UnsupportedVertexFormat(BufferFieldFormat),

    #[error("No valid surface formats found")]
    NoValidSurfaceFormat,

    // Possibly not needed
    #[error("No valid swap present mode found")]
    NoValidPresentMode,

    #[error("Error loading vulkan library: {0}")]
    LoadingError(#[from] LoadingError),

    #[error("Vulkan API error: {0}")]
    VulkanAPIError(#[from] vk::Result),

    #[error("Nul Error: {0}")]
    NulError(#[from] NulError),

    #[error("Vulkan Error: {0}")]
    VulkanError(String),
}
impl From<HandleError> for VulkanRendererError {
    fn from(value: HandleError) -> Self {
        Self::HandleError(value)
    }
}
impl RendererInstanceError for VulkanRendererError {}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ValidationLevel {
    NoValidation,
    #[default]
    Normal,
    Verbose,
}
#[derive(Default)]
struct DebugMessengerUserData {
    errors: Vec<VulkanRendererError>,
    verbose: bool,
}
struct DebugMessengerData {
    messenger: vk::DebugUtilsMessengerEXT,
    /// Only need to hold this copy for safety
    /// Will keep Arc alive while it's still needed
    /// Might use it in the renderer to interpret caught errors frmo the messenger
    _user_data: Arc<RwLock<DebugMessengerUserData>>,
    p_user_data_ptr: *mut c_void,
}
#[derive(Default, Debug)]
struct QueueFamilyIndices {
    graphics: Option<u32>,
    present: Option<u32>,
}
struct DeviceQueues {
    graphics: Option<vk::Queue>,
    present: Option<vk::Queue>,
}
struct CommandPools {
    graphics: vk::CommandPool,
}
struct InstanceExtensions {
    create_debug_utils_messenger_ext: Option<vk::PFN_vkCreateDebugUtilsMessengerEXT>,
    destroy_debug_utils_messenger_ext: Option<vk::PFN_vkDestroyDebugUtilsMessengerEXT>,
    #[cfg(target_os = "linux")]
    create_wayland_surface_khr: vk::PFN_vkCreateWaylandSurfaceKHR,
    #[cfg(target_os = "windows")]
    create_win32_surface_khr: vk::PFN_vkCreateWin32SurfaceKHR,
    destroy_surface_khr: vk::PFN_vkDestroySurfaceKHR,
    get_physical_device_surface_support_khr: vk::PFN_vkGetPhysicalDeviceSurfaceSupportKHR,
    get_physical_device_surface_capabilities_khr: vk::PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
    get_physical_device_surface_formats_khr: vk::PFN_vkGetPhysicalDeviceSurfaceFormatsKHR,
    get_physical_device_surface_present_modes_khr:
        vk::PFN_vkGetPhysicalDeviceSurfacePresentModesKHR,
    create_swapchain_khr: vk::PFN_vkCreateSwapchainKHR,
    destroy_swapchain_khr: vk::PFN_vkDestroySwapchainKHR,
    get_swapchain_images_khr: vk::PFN_vkGetSwapchainImagesKHR,

    acquire_next_image_khr: vk::PFN_vkAcquireNextImageKHR,
    queue_present_khr: vk::PFN_vkQueuePresentKHR,
}
#[derive(Default, Debug)]
struct SwapchainData {
    swapchain: vk::SwapchainKHR,
    extent: vk::Extent2D,
    #[expect(dead_code, reason = "Kept for debugging, not needed after creation")]
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    framebuffers: Vec<vk::Framebuffer>,
}

type CreateSwapchainFn =
    Box<dyn Fn(&VulkanRenderer, &WindowDataGLFW) -> Result<SwapchainData, VulkanRendererError>>;

struct VulkanShaderData {
    shader_module: vk::ShaderModule,
    stage: ShaderStage,
    #[expect(
        dead_code,
        reason = "Shader name might be used for search or debug later"
    )]
    name: CString,
}
struct VulkanBufferData {
    usage: BufferUsage,
    binding_description: vk::VertexInputBindingDescription,
    attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
    buffer: vk::Buffer,
    device_memory: vk::DeviceMemory,
}
struct VulkanPipelineData {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

struct FlowControl {
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

pub struct VulkanRenderer {
    window: Rc<RefCell<Window>>,
    framebuffer_resized: Rc<RefCell<Option<Vec2i32>>>,

    destroyed: bool,
    _entry: Entry,
    instance: Instance,
    instance_extensions: InstanceExtensions,
    physical_device: vk::PhysicalDevice,
    device: Device,
    device_queues: DeviceQueues,
    queue_family_indices: QueueFamilyIndices,
    command_pools: CommandPools,
    command_buffers: HashMap<vk::CommandPool, Vec<vk::CommandBuffer>>,
    swapchain_details: SwapchainSupportDetails,
    swapchain: SwapchainData,
    create_swapchain_fn: CreateSwapchainFn,
    surface: vk::SurfaceKHR,
    debug_messenger: Option<DebugMessengerData>,

    /// Map of Program( `Hash<shaders>` ) to associated Pipelines( `[Hash<shaders+settings>]` )
    /// Each program can have multiple pipelines
    programs: HashMap<ProgramHandle, (Vec<ShaderHandle>, Vec<usize>)>,
    pipelines: HashMap<usize, VulkanPipelineData>,
    render_pass: vk::RenderPass,
    shaders: HandleMap<ShaderHandle, VulkanShaderData>,
    buffers: HandleMap<BufferHandle, VulkanBufferData>,
    settings: Settings,

    flow_control: HashMap<vk::CommandBuffer, FlowControl>,
    current_frame: usize,

    buffer_bindings: usize,
    freed_buffer_bindings: VecDeque<usize>,
}
unsafe impl Send for VulkanRenderer {}
unsafe impl Sync for VulkanRenderer {}

pub(crate) const MAX_FRAMES_IN_FLIGHT: u32 = 2;

impl VulkanRenderer {
    #[cfg(feature = "GLFW")]
    pub fn new_glfw(
        name: &str,
        app_version: (u32, u32, u32),
        window: Rc<RefCell<Window>>,
        enable_validation: ValidationLevel,
    ) -> Result<Self, RendererError> {
        Ok(Self::new_glfw_impl(
            name,
            app_version,
            window,
            enable_validation,
        )?)
    }
}

impl Renderer for VulkanRenderer {
    fn name(&self) -> &'static str {
        "Vulkan"
    }

    fn destroy(&mut self) -> Result<(), RendererError> {
        unsafe { self.destroy_impl() }?;
        Ok(())
    }

    fn create_shader(&mut self, source: ShaderCreateInfo) -> Result<ShaderHandle, RendererError> {
        let mut handles = unsafe { self.create_shaders_impl(&[source])? };
        if handles.len() != 1 {
            return Err(RendererError::APIError(
                "Expected succesful shader creation to return one shader".to_string(),
            ));
        }
        Ok(handles.pop().unwrap())
    }
    fn create_shaders(
        &mut self,
        infos: &[ShaderCreateInfo],
    ) -> Result<Vec<ShaderHandle>, RendererError> {
        unsafe { Ok(self.create_shaders_impl(infos)?) }
    }
    fn destroy_shader(&mut self, shader: ShaderHandle) -> Result<(), RendererError> {
        unsafe { Ok(self.destroy_shaders_impl(&[shader])?) }
    }
    fn destroy_shaders(&mut self, shaders: &[ShaderHandle]) -> Result<(), RendererError> {
        unsafe { Ok(self.destroy_shaders_impl(shaders)?) }
    }

    fn create_program(&mut self, shaders: &[ShaderHandle]) -> Result<ProgramHandle, RendererError> {
        unsafe { Ok(self.create_program_impl(shaders)?) }
    }
    fn destroy_program(&mut self, program: ProgramHandle) -> Result<(), RendererError> {
        unsafe { Ok(self.destroy_program_impl(program)?) }
    }

    fn create_buffer(&mut self, data: BufferCreateInfo) -> Result<BufferHandle, RendererError> {
        unsafe { Ok(self.create_buffer_impl(data)?) }
    }
    fn destroy_buffer(&mut self, vertex_buffer: BufferHandle) -> Result<(), RendererError> {
        unsafe { Ok(self.destroy_buffer_impl(vertex_buffer)?) }
    }

    fn execute(&mut self, draw_call: DrawCall) -> Result<(), RendererError> {
        unsafe { Ok(self.execute_impl(draw_call)?) }
    }
}
