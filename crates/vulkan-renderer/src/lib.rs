use ash::{
    Entry,
    Instance,
    LoadingError,
    // Should only be `self` and `TaggedStructure`
    // Everything in `vk::` should use explicit paths to be descriptive
    vk::{self, TaggedStructure},
};
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

    #[error("Cannot enable validation layers because they are not supported")]
    ValidationLayersNotSupported,

    #[error("Failed to create debug messenger")]
    InvalidDebugMessenger,

    #[error("There are no physical devices that support vulkan")]
    NoSupportedDevices,

    #[error("Error loading vulkan library: {0}")]
    LoadingError(#[from] LoadingError),

    #[error("Vulkan API error: {0}")]
    VulkanAPIError(#[from] ash::vk::Result),

    #[error("Vulkan Error: {0}")]
    VulkanError(String),
}
impl RendererInstanceError for VulkanRendererError {}

#[derive(Default)]
struct DebugMessengerUserData {
    errors: Vec<VulkanRendererError>,
    verbose: bool,
}
struct DebugMessengerData {
    messenger: vk::DebugUtilsMessengerEXT,
    #[expect(dead_code, reason = "Not used, but should keep track of nonetheless")]
    create_func: vk::PFN_vkCreateDebugUtilsMessengerEXT,
    destroy_func: vk::PFN_vkDestroyDebugUtilsMessengerEXT,
    /// Only need to hold this copy for safety
    /// Will keep Arc alive while it's still needed
    /// Might use it in the renderer to interpret caught errors frmo the messenger
    _user_data: Arc<RwLock<DebugMessengerUserData>>,
    p_user_data_ptr: *mut c_void,
}
#[derive(Default)]
struct QueueFamilies {
    graphics: Option<i32>,
}
pub struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,
    debug_messenger: Option<DebugMessengerData>,
}
unsafe impl Send for VulkanRenderer {}
unsafe impl Sync for VulkanRenderer {}

impl VulkanRenderer {
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

    unsafe fn create_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> Result<DebugMessengerData, VulkanRendererError> {
        use vk::DebugUtilsMessengerCreateInfoEXT;
        unsafe {
            use std::ptr::null;

            let vk_instance = instance.handle();

            let user_data = Arc::<RwLock<DebugMessengerUserData>>::default();
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

            let create_func_name = c"vkCreateDebugUtilsMessengerEXT";
            let Some(create_func) = entry
                .get_instance_proc_addr(vk_instance, create_func_name.as_ptr())
                .map(|func| std::mem::transmute::<_, vk::PFN_vkCreateDebugUtilsMessengerEXT>(func))
            else {
                eprintln!("Error loading debug messenger create function");
                return Err(VulkanRendererError::InvalidDebugMessenger);
            };

            let destroy_func_name = c"vkDestroyDebugUtilsMessengerEXT";
            let Some(destroy_func) = entry
                .get_instance_proc_addr(vk_instance, destroy_func_name.as_ptr())
                .map(|func| {
                    std::mem::transmute::<_, vk::PFN_vkDestroyDebugUtilsMessengerEXT>(func)
                })
            else {
                eprintln!("Error loading debug messenger destroy function");
                return Err(VulkanRendererError::InvalidDebugMessenger);
            };

            let mut messenger = std::mem::MaybeUninit::<vk::DebugUtilsMessengerEXT>::uninit();

            let res = (create_func)(
                vk_instance,
                &debug_messanger_create_info as *const DebugUtilsMessengerCreateInfoEXT,
                null(),
                messenger.as_mut_ptr(),
            );
            if res.result().is_err() {
                return Err(VulkanRendererError::InvalidDebugMessenger);
            }

            Ok(DebugMessengerData {
                messenger: messenger.assume_init(),
                create_func,
                destroy_func,
                _user_data: user_data,
                p_user_data_ptr: p_user_data,
            })
        }
    }

    unsafe fn get_device_suitability_score(
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<i32, VulkanRendererError> {
        unsafe {
            let device_properties = instance.get_physical_device_properties(device);
            #[expect(unused_variables, reason = "No feature checks yet")]
            let device_features = instance.get_physical_device_features(device);

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
                return Ok(0);
            };

            let queue_families = Self::find_queue_families(instance, device)?;
            if queue_families.graphics.is_none() {
                return Ok(0);
            }

            Ok(score)
        }
    }

    unsafe fn find_queue_families(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<QueueFamilies, VulkanRendererError> {
        unsafe {
            let queue_families =
                instance.get_physical_device_queue_family_properties(physical_device);
            let mut res = QueueFamilies::default();

            for (index, queue_family) in queue_families.iter().enumerate() {
                if (queue_family.queue_flags & vk::QueueFlags::GRAPHICS).as_raw() != 0 {
                    res.graphics = Some(index as i32);
                }
            }

            Ok(res)
        }
    }

    unsafe fn create_physical_device(instance: &Instance) -> Result<(), VulkanRendererError> {
        unsafe {
            let device = instance
                .enumerate_physical_devices()?
                .into_iter()
                .map(|device| {
                    Ok((
                        Self::get_device_suitability_score(instance, device)?,
                        device,
                    ))
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?
                .into_iter()
                .filter(|(suitability, _)| *suitability > 0)
                .fold((0i32, None), |acc, device| {
                    if device.0 > acc.0 {
                        (device.0, Some(device.1))
                    } else {
                        acc
                    }
                });
            if let Some(device) = device.1 {
            } else {
                return Err(VulkanRendererError::NoSupportedDevices);
            }

            Ok(())
        }
    }

    #[cfg(feature = "GLFW")]
    pub fn new_glfw_impl(
        name: &str,
        app_version: (u32, u32, u32),
        window: &WindowDataGLFW,
        enable_validation: bool,
    ) -> Result<Self, VulkanRendererError> {
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
            engine_version: vk::make_api_version(0, app_version.0, app_version.1, app_version.2),
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let required_extensions = window
            .glfw
            .get_required_instance_extensions()
            .ok_or_else(|| VulkanRendererError::GLFWInstanceError)?;
        let mut required_extensions = required_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        unsafe {
            if enable_validation {
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
            if enable_validation {
                create_info.enabled_layer_count = validation_layers.len() as u32;
                create_info.pp_enabled_layer_names = layer_names.as_ptr();
            }

            let instance = entry.create_instance(&create_info, None)?;

            let debug_messenger = if enable_validation {
                Some(Self::create_debug_messenger(&entry, &instance)?)
            } else {
                None
            };

            Self::create_physical_device(&instance)?;

            Ok(Self {
                _entry: entry,
                instance,
                debug_messenger,
            })
        }
    }
    #[cfg(feature = "GLFW")]
    pub fn new_glfw(
        name: &str,
        app_version: (u32, u32, u32),
        window: &WindowDataGLFW,
        enable_validation: bool,
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
            unsafe { (messenger.destroy_func)(self.instance.handle(), messenger.messenger, null()) }

            // This should drop the pointer kept by the messenger as p_user_data
            let _ = unsafe {
                Box::from_raw(
                    messenger.p_user_data_ptr as *mut Arc<RwLock<Vec<VulkanRendererError>>>,
                )
            };
            messenger.p_user_data_ptr = null::<c_void>() as *mut c_void;
        }
        self.debug_messenger = None;
        unsafe { self.instance.destroy_instance(None) };
    }
}
