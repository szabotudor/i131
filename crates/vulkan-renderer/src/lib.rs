use ash::{Device, Entry, Instance, LoadingError, vk};
#[cfg(feature = "GLFW")]
use raw_window_handle::HandleError;
#[cfg(feature = "GLFW")]
use renderer131::RendererError;
use renderer131::{
    ProgramHandle, Renderer, RendererInstanceError, Settings, ShaderCreateInfo, ShaderHandle,
    ShaderStage,
};
use std::{
    collections::HashMap,
    ffi::{CString, NulError, c_void},
    sync::{Arc, RwLock},
};
use thiserror::Error;
#[cfg(feature = "GLFW")]
use window131::WindowDataGLFW;

use crate::vulkan_init::SwapchainSupportDetails;

pub mod build_tools;
mod vulkan_impl;
mod vulkan_init;

#[derive(Error, Debug)]
pub enum VulkanRendererError {
    #[cfg(feature = "GLFW")]
    #[error("Error getting GLFW instance")]
    GLFWInstanceError,

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

    #[error("There are no physical devices that support vulkan")]
    NoSupportedDevices,

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
#[derive(Default)]
struct SwapchainData {
    #[expect(dead_code, reason = "Saved after creation for debugging")]
    swapchain_details: SwapchainSupportDetails,
    swapchain: vk::SwapchainKHR,
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    #[expect(dead_code, reason = "Saved after creation for debugging")]
    present_mode: vk::PresentModeKHR,
    #[expect(dead_code, reason = "Saved after creation for debugging")]
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
}

struct VulkanShaderData {
    shader_module: vk::ShaderModule,
    stage: ShaderStage,
    #[expect(
        dead_code,
        reason = "Shader name might be used for search or debug later"
    )]
    name: CString,
}
struct VulkanPipelineData {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

pub struct VulkanRenderer {
    destroyed: bool,
    _entry: Entry,
    instance: Instance,
    instance_extensions: InstanceExtensions,
    device: Device,
    device_queues: DeviceQueues,
    command_pools: CommandPools,
    command_buffers: HashMap<vk::CommandPool, Vec<vk::CommandBuffer>>,
    swapchain: SwapchainData,
    surface: vk::SurfaceKHR,
    debug_messenger: Option<DebugMessengerData>,

    /// Map of Program( `Hash<shaders>` ) to associated Pipelines( `[Hash<shaders+settings>]` )
    /// Each program can have multiple pipelines
    programs: HashMap<ProgramHandle, (Vec<ShaderHandle>, Vec<usize>)>,
    pipelines: HashMap<usize, VulkanPipelineData>,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    shader_handles: usize,
    shaders: HashMap<ShaderHandle, VulkanShaderData>,
    settings: Settings,

    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
}
unsafe impl Send for VulkanRenderer {}
unsafe impl Sync for VulkanRenderer {}

impl VulkanRenderer {
    #[cfg(feature = "GLFW")]
    pub fn new_glfw(
        name: &str,
        app_version: (u32, u32, u32),
        window: &WindowDataGLFW,
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

    fn execute(&mut self, program: ProgramHandle) -> Result<(), RendererError> {
        unsafe { Ok(self.execute_impl(program)?) }
    }
}
