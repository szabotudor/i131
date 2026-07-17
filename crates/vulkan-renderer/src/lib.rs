use ash::{
    Device, Entry, Instance, LoadingError,
    vk::{self, TaggedStructure},
};
#[cfg(feature = "GLFW")]
use raw_window_handle::HandleError;
#[cfg(target_os = "linux")]
use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};
use renderer131::{Renderer, RendererError, RendererInstanceError};
use std::{
    ffi::{CStr, c_void},
    ptr::{null, null_mut},
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

    #[error("No valid surface formats found")]
    NoValidSurfaceFormat,

    // Possibly not needed
    #[error("No valid swap present mode found")]
    NoValidPresentMode,

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
struct QueueFamilyIndices {
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
    get_physical_device_surface_capabilities_khr: vk::PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
    get_physical_device_surface_formats_khr: vk::PFN_vkGetPhysicalDeviceSurfaceFormatsKHR,
    get_physical_device_surface_present_modes_khr:
        vk::PFN_vkGetPhysicalDeviceSurfacePresentModesKHR,
    create_swapchain_khr: vk::PFN_vkCreateSwapchainKHR,
    destroy_swapchain_khr: vk::PFN_vkDestroySwapchainKHR,
    get_swapchain_images_khr: vk::PFN_vkGetSwapchainImagesKHR,
}
#[derive(Default)]
struct SwapchainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}
#[derive(Default)]
struct SwapchainData {
    swapchain_details: SwapchainSupportDetails,
    swapchain: vk::SwapchainKHR,
    format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
}
pub struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,
    instance_extensions: InstanceExtensions,
    device: Device,
    device_queues: DeviceQueues,
    swapchain: SwapchainData,
    surface: vk::SurfaceKHR,
    debug_messenger: Option<DebugMessengerData>,
}
unsafe impl Send for VulkanRenderer {}
unsafe impl Sync for VulkanRenderer {}

struct CreateSwapchainArgs<'a> {
    instance_extensions: &'a InstanceExtensions,
    device: vk::Device,
    swapchain_details: &'a SwapchainSupportDetails,
    queue_family_indices: &'a QueueFamilyIndices,
    surface: vk::SurfaceKHR,
    format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    extent: vk::Extent2D,
}

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
                Self::load_instance_proc_addr(entry, instance, c"vkCreateDebugUtilsMessengerEXT")?;
            let destroy_debug_utils_messenger_ext =
                Self::load_instance_proc_addr(entry, instance, c"vkDestroyDebugUtilsMessengerEXT")?;

            let create_wayland_surface_khr =
                Self::load_instance_proc_addr(entry, instance, c"vkCreateWaylandSurfaceKHR")?;

            let destroy_surface_khr =
                Self::load_instance_proc_addr(entry, instance, c"vkDestroySurfaceKHR")?;

            let get_physical_device_surface_support_khr = Self::load_instance_proc_addr(
                entry,
                instance,
                c"vkGetPhysicalDeviceSurfaceSupportKHR",
            )?;
            let get_physical_device_surface_capabilities_khr = Self::load_instance_proc_addr(
                entry,
                instance,
                c"vkGetPhysicalDeviceSurfaceCapabilitiesKHR",
            )?;
            let get_physical_device_surface_formats_khr = Self::load_instance_proc_addr(
                entry,
                instance,
                c"vkGetPhysicalDeviceSurfaceFormatsKHR",
            )?;
            let get_physical_device_surface_present_modes_khr = Self::load_instance_proc_addr(
                entry,
                instance,
                c"vkGetPhysicalDeviceSurfacePresentModesKHR",
            )?;

            let create_swapchain_khr =
                Self::load_instance_proc_addr(entry, instance, c"vkCreateSwapchainKHR")?;
            let destroy_swapchain_khr =
                Self::load_instance_proc_addr(entry, instance, c"vkDestroySwapchainKHR")?;
            let get_swapchain_images_khr =
                Self::load_instance_proc_addr(entry, instance, c"vkGetSwapchainImagesKHR")?;

            Ok(InstanceExtensions {
                create_debug_utils_messenger_ext,
                destroy_debug_utils_messenger_ext,
                create_wayland_surface_khr,
                destroy_surface_khr,
                get_physical_device_surface_support_khr,
                get_physical_device_surface_capabilities_khr,
                get_physical_device_surface_formats_khr,
                get_physical_device_surface_present_modes_khr,
                create_swapchain_khr,
                destroy_swapchain_khr,
                get_swapchain_images_khr,
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
    ) -> Result<QueueFamilyIndices, VulkanRendererError> {
        unsafe {
            let queue_families =
                instance.get_physical_device_queue_family_properties(physical_device);
            let mut res = QueueFamilyIndices::default();

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

    unsafe fn get_swapchain_details(
        instance_extensions: &InstanceExtensions,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<SwapchainSupportDetails, VulkanRendererError> {
        unsafe {
            let mut surface_capabilities =
                std::mem::MaybeUninit::<vk::SurfaceCapabilitiesKHR>::uninit();

            // Read capabilities
            (instance_extensions.get_physical_device_surface_capabilities_khr)(
                physical_device,
                surface,
                surface_capabilities.as_mut_ptr(),
            )
            .result()?;

            // Read supported formats
            let mut format_count = 0u32;
            (instance_extensions.get_physical_device_surface_formats_khr)(
                physical_device,
                surface,
                &mut format_count as *mut u32,
                null_mut::<vk::SurfaceFormatKHR>(),
            )
            .result()?;

            let mut formats = vec![vk::SurfaceFormatKHR::default(); format_count as usize];
            (instance_extensions.get_physical_device_surface_formats_khr)(
                physical_device,
                surface,
                &mut format_count as *mut u32,
                formats.as_mut_ptr(),
            )
            .result()?;

            // Read supported present modes
            let mut present_mode_count = 0u32;
            (instance_extensions.get_physical_device_surface_present_modes_khr)(
                physical_device,
                surface,
                &mut present_mode_count as *mut u32,
                null_mut::<vk::PresentModeKHR>(),
            )
            .result()?;

            let mut present_modes =
                vec![vk::PresentModeKHR::default(); present_mode_count as usize];
            (instance_extensions.get_physical_device_surface_present_modes_khr)(
                physical_device,
                surface,
                &mut present_mode_count as *mut u32,
                present_modes.as_mut_ptr(),
            )
            .result()?;

            Ok(SwapchainSupportDetails {
                capabilities: surface_capabilities.assume_init(),
                formats,
                present_modes,
            })
        }
    }

    unsafe fn get_device_suitability_score(
        instance: &Instance,
        instance_extensions: &InstanceExtensions,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<(i32, QueueFamilyIndices, SwapchainSupportDetails), VulkanRendererError> {
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

            let swapchain_details =
                Self::get_swapchain_details(instance_extensions, physical_device, surface)?;

            if swapchain_details.formats.is_empty() || swapchain_details.present_modes.is_empty() {
                return Ok((
                    0,
                    QueueFamilyIndices::default(),
                    SwapchainSupportDetails::default(),
                ));
            }

            if !device_supports_required_extensions {
                return Ok((
                    0,
                    QueueFamilyIndices::default(),
                    SwapchainSupportDetails::default(),
                ));
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
                return Ok((
                    0,
                    QueueFamilyIndices::default(),
                    SwapchainSupportDetails::default(),
                ));
            };

            let queue_family_indices =
                Self::find_queue_families(instance, instance_extensions, physical_device, surface)?;
            if queue_family_indices.graphics.is_none() {
                return Ok((
                    0,
                    QueueFamilyIndices::default(),
                    SwapchainSupportDetails::default(),
                ));
            }

            Ok((score, queue_family_indices, swapchain_details))
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
    ) -> Result<
        (
            vk::PhysicalDevice,
            Device,
            QueueFamilyIndices,
            DeviceQueues,
            SwapchainSupportDetails,
        ),
        VulkanRendererError,
    > {
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
                .filter(
                    |(
                        (suitability_score, _queue_family_indices, _swapchain_details),
                        _physical_device,
                    )| { *suitability_score > 0 },
                )
                // Find the one device with the best score.
                //
                // If there are multiple devices with the largest score, the first is chosen
                .fold(
                    // Default empty device
                    (
                        (
                            0i32,
                            QueueFamilyIndices::default(),
                            SwapchainSupportDetails::default(),
                        ),
                        None,
                    ),
                    // TODO: Difficult to understand, should organize the fold function
                    |acc, device| {
                        let (
                            (
                                acc_device_suitability_score,
                                acc_queue_family_indices,
                                acc_swapchain_details,
                            ),
                            acc_physical_device,
                        ) = acc;

                        let (
                            (device_suitability_score, queue_family_indices, swapchain_details),
                            physical_device,
                        ) = device;

                        if device_suitability_score > acc_device_suitability_score {
                            (
                                (
                                    device_suitability_score,
                                    queue_family_indices,
                                    swapchain_details,
                                ),
                                Some(physical_device),
                            )
                        } else {
                            (
                                (
                                    acc_device_suitability_score,
                                    acc_queue_family_indices,
                                    acc_swapchain_details,
                                ),
                                acc_physical_device,
                            )
                        }
                    },
                );
            if let Some(physical_device) = device.1 {
                let queue_family_indices = device.0.1;
                let swapchain_details = device.0.2;
                let queue_priority = 1.0f32;

                let queue_create_infos = [
                    vk::DeviceQueueCreateInfo {
                        s_type: vk::DeviceQueueCreateInfo::STRUCTURE_TYPE,
                        queue_family_index: queue_family_indices.graphics.ok_or_else(|| {
                            VulkanRendererError::MissingQueue("GRAPHICS".to_string())
                        })?,
                        queue_count: 1,
                        p_queue_priorities: &queue_priority as *const f32,
                        ..Default::default()
                    },
                    vk::DeviceQueueCreateInfo {
                        s_type: vk::DeviceQueueCreateInfo::STRUCTURE_TYPE,
                        queue_family_index: queue_family_indices.present.ok_or_else(|| {
                            VulkanRendererError::MissingQueue("PRESENT".to_string())
                        })?,
                        queue_count: 1,
                        p_queue_priorities: &queue_priority as *const f32,
                        ..Default::default()
                    },
                ];

                let required_extensions = Self::REQUIRED_DEVICE_EXTENSIONS
                    .map(|ext| ext.as_ptr())
                    .into_iter()
                    .collect::<Vec<_>>();
                let create_info = vk::DeviceCreateInfo {
                    s_type: vk::DeviceCreateInfo::STRUCTURE_TYPE,
                    p_queue_create_infos: queue_create_infos.as_ptr(),
                    queue_create_info_count: queue_create_infos.len() as u32,
                    p_enabled_features: null(),
                    enabled_extension_count: required_extensions.len() as u32,
                    pp_enabled_extension_names: required_extensions.as_ptr(),
                    ..Default::default()
                };

                let device = instance.create_device(physical_device, &create_info, None)?;

                let device_queues = DeviceQueues {
                    graphics: queue_family_indices
                        .graphics
                        .map(|idx| device.get_device_queue(idx, 0)),
                    present: queue_family_indices
                        .present
                        .map(|idx| device.get_device_queue(idx, 0)),
                };

                Ok((
                    physical_device,
                    device,
                    queue_family_indices,
                    device_queues,
                    swapchain_details,
                ))
            } else {
                Err(VulkanRendererError::NoSupportedDevices)
            }
        }
    }

    fn choose_swapchain_format(
        formats: &[vk::SurfaceFormatKHR],
    ) -> Result<vk::SurfaceFormatKHR, VulkanRendererError> {
        if formats.is_empty() {
            return Err(VulkanRendererError::NoValidSurfaceFormat);
        }
        let preferred = vk::SurfaceFormatKHR {
            format: vk::Format::R8G8B8A8_SRGB,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        };

        if formats.contains(&preferred) {
            return Ok(preferred);
        }

        Ok(formats[0])
    }
    fn choose_swap_present_mode(
        present_modes: &[vk::PresentModeKHR],
    ) -> Result<vk::PresentModeKHR, VulkanRendererError> {
        if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            return Ok(vk::PresentModeKHR::MAILBOX);
        }

        Ok(vk::PresentModeKHR::FIFO)
    }
    fn choose_swap_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        current_extent: vk::Extent2D,
    ) -> Result<vk::Extent2D, VulkanRendererError> {
        if capabilities.current_extent.width != u32::MAX {
            Ok(capabilities.current_extent)
        } else {
            Ok(vk::Extent2D {
                width: math131::util::clamp(
                    current_extent.width,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: math131::util::clamp(
                    current_extent.height,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            })
        }
    }

    unsafe fn create_swapchain(
        args: CreateSwapchainArgs,
    ) -> Result<(vk::SwapchainKHR, Vec<vk::Image>), VulkanRendererError> {
        unsafe {
            let CreateSwapchainArgs {
                instance_extensions,
                device,
                swapchain_details,
                queue_family_indices,
                surface,
                format,
                present_mode,
                extent,
            } = args;

            let mut image_count = swapchain_details.capabilities.min_image_count + 1;

            if swapchain_details.capabilities.max_image_count != 0
                && image_count > swapchain_details.capabilities.max_image_count
            {
                image_count = swapchain_details.capabilities.max_image_count;
            }

            let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
                s_type: vk::SwapchainCreateInfoKHR::STRUCTURE_TYPE,
                surface,
                min_image_count: image_count,
                image_format: format.format,
                image_color_space: format.color_space,
                image_extent: extent,
                image_array_layers: 1,
                image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                pre_transform: swapchain_details.capabilities.current_transform,
                composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                present_mode,
                clipped: vk::TRUE,
                old_swapchain: vk::SwapchainKHR::null(),
                ..Default::default()
            };

            let Some((graphics_queue, present_queue)) = queue_family_indices
                .graphics
                .zip(queue_family_indices.present)
            else {
                return Err(VulkanRendererError::MissingQueue(
                    "PRESENT or GRAPHICS".to_string(),
                ));
            };

            let queue_family_indices_array = [graphics_queue, present_queue];

            if graphics_queue != present_queue {
                swapchain_create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
                swapchain_create_info.queue_family_index_count = 2;
                swapchain_create_info.p_queue_family_indices = queue_family_indices_array.as_ptr();
            } else {
                swapchain_create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
                swapchain_create_info.queue_family_index_count = 0;
                swapchain_create_info.p_queue_family_indices = null();
            }

            let mut swapchain = vk::SwapchainKHR::default();

            (instance_extensions.create_swapchain_khr)(
                device,
                &swapchain_create_info as *const _,
                null(),
                &mut swapchain as *mut vk::SwapchainKHR,
            )
            .result()?;

            let mut swapchain_image_count = 0u32;
            (instance_extensions.get_swapchain_images_khr)(
                device,
                swapchain,
                &mut swapchain_image_count as *mut u32,
                null_mut(),
            )
            .result()?;

            let mut swapchain_images = vec![vk::Image::default(); swapchain_image_count as usize];
            (instance_extensions.get_swapchain_images_khr)(
                device,
                swapchain,
                &mut swapchain_image_count as *mut u32,
                swapchain_images.as_mut_ptr(),
            )
            .result()?;

            Ok((swapchain, swapchain_images))
        }
    }

    unsafe fn create_swapchain_image_views(
        device: &Device,
        swapchain_format: vk::SurfaceFormatKHR,
        swapchain_images: &[vk::Image],
    ) -> Result<Vec<vk::ImageView>, VulkanRendererError> {
        unsafe {
            let image_views = swapchain_images
                .iter()
                .map(|image| {
                    let image_view_create_info = vk::ImageViewCreateInfo {
                        s_type: vk::ImageViewCreateInfo::STRUCTURE_TYPE,
                        image: *image,
                        view_type: vk::ImageViewType::TYPE_2D,
                        format: swapchain_format.format,
                        components: vk::ComponentMapping {
                            r: vk::ComponentSwizzle::IDENTITY,
                            g: vk::ComponentSwizzle::IDENTITY,
                            b: vk::ComponentSwizzle::IDENTITY,
                            a: vk::ComponentSwizzle::IDENTITY,
                        },
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        ..Default::default()
                    };

                    let image_view = device.create_image_view(&image_view_create_info, None)?;
                    Ok(image_view)
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?;

            Ok(image_views)
        }
    }

    #[cfg(feature = "GLFW")]
    unsafe fn create_swapchain_glfw(
        instance_extensions: &InstanceExtensions,
        device: &Device,
        swapchain_details: SwapchainSupportDetails,
        surface: vk::SurfaceKHR,
        queue_family_indices: &QueueFamilyIndices,
        window: &WindowDataGLFW,
    ) -> Result<SwapchainData, VulkanRendererError> {
        unsafe {
            let window_extent = window.window.get_framebuffer_size();
            let window_extent = vk::Extent2D {
                width: window_extent.0 as u32,
                height: window_extent.1 as u32,
            };

            let format = Self::choose_swapchain_format(&swapchain_details.formats)?;
            let present_mode = Self::choose_swap_present_mode(&swapchain_details.present_modes)?;
            let extent = Self::choose_swap_extent(&swapchain_details.capabilities, window_extent)?;

            let (swapchain, swapchain_images) = Self::create_swapchain(CreateSwapchainArgs {
                instance_extensions,
                device: device.handle(),
                swapchain_details: &swapchain_details,
                queue_family_indices,
                surface,
                format,
                present_mode,
                extent,
            })?;

            let swapchain_image_views =
                Self::create_swapchain_image_views(device, format, &swapchain_images)?;

            Ok(SwapchainData {
                swapchain_details,
                swapchain,
                format,
                present_mode,
                swapchain_images,
                swapchain_image_views,
            })
        }
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

            let (_physical_device, device, queue_family_indices, device_queues, swapchain_details) =
                Self::create_device(&instance, &instance_extensions, surface)?;

            let swapchain = Self::create_swapchain_glfw(
                &instance_extensions,
                &device,
                swapchain_details,
                surface,
                &queue_family_indices,
                window,
            )?;

            Ok(Self {
                _entry: entry,
                instance,
                instance_extensions,
                device,
                device_queues,
                swapchain,
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
            for image_view in &self.swapchain.swapchain_image_views {
                self.device.destroy_image_view(*image_view, None);
            }
            self.swapchain.swapchain_image_views.clear();

            (self.instance_extensions.destroy_swapchain_khr)(
                self.device.handle(),
                self.swapchain.swapchain,
                null(),
            );
            self.swapchain = SwapchainData::default();

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
