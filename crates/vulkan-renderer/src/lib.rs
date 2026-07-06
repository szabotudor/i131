use ash::{
    Device,
    Entry,
    Instance,
    LoadingError,
    // Should only be `self` and `TaggedStructure`
    // Everything in `vk::` should use explicit paths to be descriptive
    vk::{self, TaggedStructure},
};
#[cfg(feature = "GLFW")]
use raw_window_handle::HandleError;
#[cfg(target_os = "linux")]
use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};
use renderer131::{Renderer, RendererError, RendererInstanceError};
use std::{
    ffi::{CStr, c_void},
    ptr::null,
    sync::{Arc, RwLock},
};
use thiserror::Error;
use window131::WindowDataGLFW;

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

    #[error("Error loading vulkan library: {0}")]
    LoadingError(#[from] LoadingError),

    #[error("Vulkan API error: {0}")]
    VulkanAPIError(#[from] vk::Result),

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
struct QueueFamilies {
    graphics: Option<u32>,
    present: Option<u32>,
}
struct DeviceQueues {
    graphics: Option<vk::Queue>,
    present: Option<vk::Queue>,
}
struct InstanceExtensions {
    create_debug_utils_messenger_ext: vk::PFN_vkCreateDebugUtilsMessengerEXT,
    destroy_debug_utils_messenger_ext: vk::PFN_vkDestroyDebugUtilsMessengerEXT,
    #[cfg(target_os = "linux")]
    create_wayland_surface_khr: vk::PFN_vkCreateWaylandSurfaceKHR,
    destroy_surface_khr: vk::PFN_vkDestroySurfaceKHR,
    get_physical_device_surface_support_khr: vk::PFN_vkGetPhysicalDeviceSurfaceSupportKHR,
}
struct SwapchainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}
pub struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,
    instance_extensions: InstanceExtensions,
    device: Device,
    device_queues: DeviceQueues,
    swapchain_details: SwapchainSupportDetails,
    surface: vk::SurfaceKHR,
    debug_messenger: Option<DebugMessengerData>,
}
unsafe impl Send for VulkanRenderer {}
unsafe impl Sync for VulkanRenderer {}

impl VulkanRenderer {
    const REQUIRED_DEVICE_EXTENSIONS: &[&CStr; 1] = &[vk::KHR_SWAPCHAIN_NAME];

    unsafe extern "system" fn debug_message_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
        p_user_data: *mut c_void,
    ) -> vk::Bool32 {
        let p_user_data =
            unsafe { Box::<Arc<RwLock<DebugMessengerUserData>>>::from_raw(p_user_data as *mut _) };
        let user_data = p_user_data.as_ref().clone();
        let mut user_data = match user_data.write() {
            Ok(ok) => ok,
            Err(_) => return vk::FALSE,
        };

        #[expect(
            unused_must_use,
            reason = "p_user_data pointer still exists and will be sent to this function again"
        )]
        Box::into_raw(p_user_data);

        // TODO: Handle message severity, and maybe send error data to the renderer struct somehow
        //
        // Maybe shared Arc? Safe to send via raw pointer p_user_data?
        unsafe {
            let data = &*p_callback_data;
            let message = match CStr::from_ptr(data.p_message).to_str() {
                Ok(ok) => ok,
                Err(_) => return false.into(),
            };

            match message_severity {
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
                    eprintln!("Vulkan: {message}");
                }
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
                    eprintln!("Vulkan Warning: {message}");
                }
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
                    eprintln!("Vulkan Error: {message}");
                    user_data
                        .errors
                        .push(VulkanRendererError::VulkanError(message.to_string()));
                }
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
                    if user_data.verbose {
                        eprintln!("Vulkan Error: {message}");
                    }
                }
                _ => {}
            }
            true.into()
        }
    }

    unsafe fn load_instance_proc_addr<T: Sized>(
        entry: &Entry,
        instance: vk::Instance,
        name: &CStr,
    ) -> Result<T, VulkanRendererError> {
        unsafe {
            let Some(proc_addr) = entry
                .get_instance_proc_addr(instance, name.as_ptr())
                .map(|f| std::mem::transmute_copy(&f))
            else {
                return Err(VulkanRendererError::MissingExtention(
                    "vkCreateWaylandSurfaceKHR".to_string(),
                ));
            };

            Ok(proc_addr)
        }
    }
    unsafe fn load_instance_extensions(
        entry: &Entry,
        instance: vk::Instance,
    ) -> Result<InstanceExtensions, VulkanRendererError> {
        unsafe {
            let create_debug_utils_messenger_ext =
                Self::load_instance_proc_addr::<vk::PFN_vkCreateDebugUtilsMessengerEXT>(
                    entry,
                    instance,
                    c"vkCreateDebugUtilsMessengerEXT",
                )?;
            let destroy_debug_utils_messenger_ext =
                Self::load_instance_proc_addr::<vk::PFN_vkDestroyDebugUtilsMessengerEXT>(
                    entry,
                    instance,
                    c"vkDestroyDebugUtilsMessengerEXT",
                )?;

            let create_wayland_surface_khr = Self::load_instance_proc_addr::<
                vk::PFN_vkCreateWaylandSurfaceKHR,
            >(
                entry, instance, c"vkCreateWaylandSurfaceKHR"
            )?;

            let destroy_surface_khr = Self::load_instance_proc_addr::<vk::PFN_vkDestroySurfaceKHR>(
                entry,
                instance,
                c"vkDestroySurfaceKHR",
            )?;

            let get_physical_device_surface_support_khr =
                Self::load_instance_proc_addr::<vk::PFN_vkGetPhysicalDeviceSurfaceSupportKHR>(
                    entry,
                    instance,
                    c"vkGetPhysicalDeviceSurfaceSupportKHR",
                )?;

            Ok(InstanceExtensions {
                create_debug_utils_messenger_ext,
                destroy_debug_utils_messenger_ext,
                create_wayland_surface_khr,
                destroy_surface_khr,
                get_physical_device_surface_support_khr,
            })
        }
    }

    unsafe fn create_instance(
        name: &str,
        app_version: (u32, u32, u32),
        mut required_extensions: Vec<*const u8>,
        enable_validation: ValidationLevel,
    ) -> Result<(Entry, Instance), VulkanRendererError> {
        unsafe {
            let app_info = vk::ApplicationInfo {
                s_type: vk::ApplicationInfo::STRUCTURE_TYPE,
                p_application_name: name as *const str as *const i8,
                application_version: vk::make_api_version(
                    0,
                    app_version.0,
                    app_version.1,
                    app_version.2,
                ),
                p_engine_name: "I131" as *const str as *const i8,
                engine_version: vk::make_api_version(
                    0,
                    app_version.0,
                    app_version.1,
                    app_version.2,
                ),
                api_version: vk::API_VERSION_1_3,
                ..Default::default()
            };

            if enable_validation != ValidationLevel::NoValidation {
                required_extensions.push(vk::EXT_DEBUG_UTILS_NAME.as_ptr() as *const u8);
            }
            let entry = Entry::linked();

            let mut create_info = vk::InstanceCreateInfo {
                s_type: vk::InstanceCreateInfo::STRUCTURE_TYPE,
                p_application_info: &app_info as *const vk::ApplicationInfo,
                enabled_extension_count: required_extensions.len() as u32,
                pp_enabled_extension_names: required_extensions.as_ptr() as *const *const i8,
                enabled_layer_count: 0,
                ..Default::default()
            };

            // TODO: Very precarious
            //
            // Arrays might get dropped before `create_instance` reads them from the ptr
            let validation_layers = [c"VK_LAYER_KHRONOS_validation"];
            let layer_names = &validation_layers.map(|layer| layer.as_ptr());
            if enable_validation != ValidationLevel::NoValidation {
                create_info.enabled_layer_count = validation_layers.len() as u32;
                create_info.pp_enabled_layer_names = layer_names.as_ptr();
            }

            let instance = entry.create_instance(&create_info, None)?;

            Ok((entry, instance))
        }
    }

    unsafe fn create_debug_messenger(
        instance: &Instance,
        instance_extensions: &InstanceExtensions,
        validation_level: ValidationLevel,
    ) -> Result<Option<DebugMessengerData>, VulkanRendererError> {
        use vk::DebugUtilsMessengerCreateInfoEXT;

        if validation_level == ValidationLevel::NoValidation {
            return Ok(None);
        }

        unsafe {
            use std::ptr::null;

            let vk_instance = instance.handle();

            let user_data = Arc::new(RwLock::new(DebugMessengerUserData {
                verbose: validation_level == ValidationLevel::Verbose,
                ..Default::default()
            }));
            let p_user_data = Box::into_raw(Box::new(user_data.clone())) as *mut c_void;

            let debug_messanger_create_info = DebugUtilsMessengerCreateInfoEXT {
                s_type: vk::DebugUtilsMessengerCreateInfoEXT::STRUCTURE_TYPE,
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                pfn_user_callback: Some(Self::debug_message_callback),
                p_user_data,
                ..Default::default()
            };

            let mut messenger = std::mem::MaybeUninit::<vk::DebugUtilsMessengerEXT>::uninit();

            let res = (instance_extensions.create_debug_utils_messenger_ext)(
                vk_instance,
                &debug_messanger_create_info as *const DebugUtilsMessengerCreateInfoEXT,
                null(),
                messenger.as_mut_ptr(),
            );
            if res.result().is_err() {
                return Err(VulkanRendererError::InvalidDebugMessenger);
            }

            Ok(Some(DebugMessengerData {
                messenger: messenger.assume_init(),
                _user_data: user_data,
                p_user_data_ptr: p_user_data,
            }))
        }
    }

    unsafe fn find_queue_families(
        instance: &Instance,
        instance_extensions: &InstanceExtensions,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<QueueFamilies, VulkanRendererError> {
        unsafe {
            let queue_families =
                instance.get_physical_device_queue_family_properties(physical_device);
            let mut res = QueueFamilies::default();

            for (index, queue_family) in queue_families.iter().enumerate() {
                if (queue_family.queue_flags & vk::QueueFlags::GRAPHICS).as_raw() != 0 {
                    res.graphics = Some(index as u32);
                }

                let mut present_support = vk::FALSE;
                (instance_extensions.get_physical_device_surface_support_khr)(
                    physical_device,
                    index as u32,
                    surface,
                    &mut present_support as *mut vk::Bool32,
                )
                .result()?;

                if present_support != 0 {
                    res.present = Some(index as u32);
                }
            }

            Ok(res)
        }
    }

    unsafe fn get_device_suitability_score(
        instance: &Instance,
        instance_extensions: &InstanceExtensions,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<(i32, QueueFamilies), VulkanRendererError> {
        unsafe {
            let device_properties = instance.get_physical_device_properties(physical_device);
            #[expect(unused_variables, reason = "No feature checks yet")]
            let device_features = instance.get_physical_device_features(physical_device);

            let device_extensions =
                instance.enumerate_device_extension_properties(physical_device)?;
            let device_supports_required_extensions =
                Self::REQUIRED_DEVICE_EXTENSIONS.iter().all(|ext| {
                    device_extensions
                        .iter()
                        .any(|device_ext| CStr::from_ptr(device_ext.extension_name.as_ptr()) == ext)
                });

            if !device_supports_required_extensions {
                return Ok((0, QueueFamilies::default()));
            }

            let score = if (device_properties.device_type.as_raw()
                & vk::PhysicalDeviceType::VIRTUAL_GPU.as_raw())
                != 0
            {
                1
            } else if (device_properties.device_type.as_raw()
                & vk::PhysicalDeviceType::INTEGRATED_GPU.as_raw())
                != 0
            {
                2
            } else if (device_properties.device_type.as_raw()
                & vk::PhysicalDeviceType::DISCRETE_GPU.as_raw())
                != 0
            {
                3
            } else {
                return Ok((0, QueueFamilies::default()));
            };

            let queue_families =
                Self::find_queue_families(instance, instance_extensions, physical_device, surface)?;
            if queue_families.graphics.is_none() {
                return Ok((0, QueueFamilies::default()));
            }

            Ok((score, queue_families))
        }
    }

    #[cfg(target_os = "linux")]
    unsafe fn create_surface_wayland(
        instance: &Instance,
        instance_extensions: &InstanceExtensions,
        window: WaylandWindowHandle,
        display: WaylandDisplayHandle,
    ) -> Result<vk::SurfaceKHR, VulkanRendererError> {
        unsafe {
            let create_info = vk::WaylandSurfaceCreateInfoKHR {
                s_type: vk::WaylandSurfaceCreateInfoKHR::STRUCTURE_TYPE,
                surface: window.surface.as_ptr(),
                display: display.display.as_ptr(),
                ..Default::default()
            };

            let mut surface = std::mem::MaybeUninit::<vk::SurfaceKHR>::uninit();

            let res = (instance_extensions.create_wayland_surface_khr)(
                instance.handle(),
                &create_info as *const _,
                null(),
                surface.as_mut_ptr(),
            );
            if res.result().is_err() {
                return Err(VulkanRendererError::SurfaceCreateFailure(
                    "Wayland".to_string(),
                ));
            }

            Ok(surface.assume_init())
        }
    }

    #[cfg(feature = "GLFW")]
    unsafe fn create_surface_glfw(
        instance: &Instance,
        instance_extensions: &InstanceExtensions,
        window: &WindowDataGLFW,
    ) -> Result<vk::SurfaceKHR, VulkanRendererError> {
        unsafe {
            use raw_window_handle::{
                HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
            };

            let handle = window.window.window_handle()?;
            let display_handle = window.window.display_handle()?;
            match (handle.as_raw(), display_handle.as_raw()) {
                #[cfg(target_os = "linux")]
                (
                    RawWindowHandle::Wayland(wayland_window_handle),
                    RawDisplayHandle::Wayland(wayland_display_handle),
                ) => Self::create_surface_wayland(
                    instance,
                    instance_extensions,
                    wayland_window_handle,
                    wayland_display_handle,
                ),
                #[cfg(target_os = "linux")]
                (
                    RawWindowHandle::Xlib(_xlib_window_handle),
                    RawDisplayHandle::Xlib(_xlib_display_handle),
                ) => {
                    todo!("Implement X11 surface creation for vulkan")
                }
                #[cfg(target_os = "windows")]
                RawWindowHandle::Win32(_win32_window_handle) => {
                    todo!("Implement Win32 surface creation for vulkan")
                }

                other => Err(VulkanRendererError::UnknownGLFWError(format!(
                    "Vulkan surface creation not implemented for: {other:?}"
                ))),
            }
        }
    }

    unsafe fn create_device(
        instance: &Instance,
        instance_extensions: &InstanceExtensions,
        surface: vk::SurfaceKHR,
    ) -> Result<(Device, DeviceQueues), VulkanRendererError> {
        unsafe {
            let device = instance
                .enumerate_physical_devices()?
                .into_iter()
                .map(|device| {
                    Ok((
                        Self::get_device_suitability_score(
                            instance,
                            instance_extensions,
                            device,
                            surface,
                        )?,
                        device,
                    ))
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?
                .into_iter()
                .filter(|(suitability, _)| suitability.0 > 0)
                .fold(((0i32, QueueFamilies::default()), None), |acc, device| {
                    if device.0.0 > acc.0.0 {
                        (device.0, Some(device.1))
                    } else {
                        acc
                    }
                });
            if let Some(physical_device) = device.1 {
                let queue_families = device.0.1;
                let queue_priority = 1.0f32;

                let queue_create_infos = [
                    vk::DeviceQueueCreateInfo {
                        s_type: vk::DeviceQueueCreateInfo::STRUCTURE_TYPE,
                        queue_family_index: queue_families.graphics.ok_or_else(|| {
                            VulkanRendererError::MissingQueue("GRAPHICS".to_string())
                        })?,
                        queue_count: 1,
                        p_queue_priorities: &queue_priority as *const f32,
                        ..Default::default()
                    },
                    vk::DeviceQueueCreateInfo {
                        s_type: vk::DeviceQueueCreateInfo::STRUCTURE_TYPE,
                        queue_family_index: queue_families.present.ok_or_else(|| {
                            VulkanRendererError::MissingQueue("PRESENT".to_string())
                        })?,
                        queue_count: 1,
                        p_queue_priorities: &queue_priority as *const f32,
                        ..Default::default()
                    },
                ];

                let create_info = vk::DeviceCreateInfo {
                    s_type: vk::DeviceCreateInfo::STRUCTURE_TYPE,
                    p_queue_create_infos: queue_create_infos.as_ptr(),
                    queue_create_info_count: queue_create_infos.len() as u32,
                    p_enabled_features: null(),
                    ..Default::default()
                };

                let device = instance.create_device(physical_device, &create_info, None)?;

                let device_queues = DeviceQueues {
                    graphics: queue_families
                        .graphics
                        .map(|idx| device.get_device_queue(idx, 0)),
                    present: queue_families
                        .present
                        .map(|idx| device.get_device_queue(idx, 0)),
                };

                Ok((device, device_queues))
            } else {
                Err(VulkanRendererError::NoSupportedDevices)
            }
        }
    }

    fn create_swapchain(
        instance: &Instance,
        device: &Device,
    ) -> Result<SwapchainSupportDetails, VulkanRendererError> {
        todo!()
    }

    #[cfg(feature = "GLFW")]
    pub fn new_glfw_impl(
        name: &str,
        app_version: (u32, u32, u32),
        window: &WindowDataGLFW,
        enable_validation: ValidationLevel,
    ) -> Result<Self, VulkanRendererError> {
        unsafe {
            let required_extensions = window
                .glfw
                .get_required_instance_extensions()
                .ok_or_else(|| VulkanRendererError::GLFWInstanceError)?;
            let required_extensions = required_extensions
                .iter()
                .map(|ext| ext.as_ptr())
                .collect::<Vec<_>>();

            let (entry, instance) =
                Self::create_instance(name, app_version, required_extensions, enable_validation)?;

            let instance_extensions = Self::load_instance_extensions(&entry, instance.handle())?;

            let debug_messenger =
                Self::create_debug_messenger(&instance, &instance_extensions, enable_validation)?;

            let surface = Self::create_surface_glfw(&instance, &instance_extensions, window)?;

            let (device, device_queues) =
                Self::create_device(&instance, &instance_extensions, surface)?;

            let swapchain_details = Self::create_swapchain(&instance, &device)?;

            Ok(Self {
                _entry: entry,
                instance,
                instance_extensions,
                device,
                device_queues,
                swapchain_details,
                surface,
                debug_messenger,
            })
        }
    }
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

impl Renderer for VulkanRenderer {}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        if let Some(messenger) = &mut self.debug_messenger {
            unsafe {
                (self.instance_extensions.destroy_debug_utils_messenger_ext)(
                    self.instance.handle(),
                    messenger.messenger,
                    null(),
                )
            }

            // This should drop the pointer kept by the messenger as p_user_data
            let _ = unsafe {
                Box::from_raw(
                    messenger.p_user_data_ptr as *mut Arc<RwLock<Vec<VulkanRendererError>>>,
                )
            };
            messenger.p_user_data_ptr = null::<c_void>() as *mut c_void;
        }
        self.debug_messenger = None;

        unsafe {
            self.device.destroy_device(None);
            (self.instance_extensions.destroy_surface_khr)(
                self.instance.handle(),
                self.surface,
                null(),
            );
            self.instance.destroy_instance(None);
        }
    }
}
