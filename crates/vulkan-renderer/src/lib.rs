use ash::{
    Entry, Instance, LoadingError,
    vk::{
        API_VERSION_1_3, ApplicationInfo, Bool32, DebugUtilsMessageSeverityFlagsEXT,
        DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerEXT,
        InstanceCreateInfo, PFN_vkCreateDebugUtilsMessengerEXT,
        PFN_vkDestroyDebugUtilsMessengerEXT, TaggedStructure, make_api_version,
    },
};
use renderer131::{Renderer, RendererError, RendererInstanceError};
use std::{
    ffi::{CStr, c_void},
    ptr::null,
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

    #[error("Error loading vulkan library: {0}")]
    LoadingError(#[from] LoadingError),

    #[error("Vulkan API error: {0}")]
    VulkanAPIError(#[from] ash::vk::Result),
}
impl RendererInstanceError for VulkanRendererError {}

struct DebugMessengerData {
    messenger: DebugUtilsMessengerEXT,
    #[expect(dead_code, reason = "Not used, but should keep track of nonetheless")]
    create_func: PFN_vkCreateDebugUtilsMessengerEXT,
    destroy_func: PFN_vkDestroyDebugUtilsMessengerEXT,
}
pub struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,
    debug_messenger: Option<DebugMessengerData>,
}

impl VulkanRenderer {
    unsafe extern "system" fn debug_message_callback(
        _message_severity: DebugUtilsMessageSeverityFlagsEXT,
        _message_types: DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const DebugUtilsMessengerCallbackDataEXT<'_>,
        _p_user_data: *mut c_void,
    ) -> Bool32 {
        // TODO: Handle message severity, and maybe send error data to the renderer struct somehow
        //
        // Maybe shared Arc? Safe to send via raw pointer p_user_data?
        unsafe {
            let data = &*p_callback_data;
            let message = CStr::from_ptr(data.p_message).to_str();
            match message {
                Ok(ok) => eprintln!("Vulkan error: {ok}"),
                Err(_) => return false.into(),
            }
            true.into()
        }
    }

    #[cfg(feature = "GLFW")]
    pub fn new_glfw_impl(
        name: &str,
        app_version: (u32, u32, u32),
        window: &WindowDataGLFW,
        enable_validation: bool,
    ) -> Result<Self, VulkanRendererError> {
        let app_info = ApplicationInfo {
            s_type: ApplicationInfo::STRUCTURE_TYPE,
            p_application_name: name as *const str as *const i8,
            application_version: make_api_version(0, app_version.0, app_version.1, app_version.2),
            p_engine_name: "I131" as *const str as *const i8,
            engine_version: make_api_version(0, app_version.0, app_version.1, app_version.2),
            api_version: API_VERSION_1_3,
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
            use std::mem::MaybeUninit;

            if enable_validation {
                use ash::vk::EXT_DEBUG_UTILS_NAME;

                required_extensions.push(EXT_DEBUG_UTILS_NAME.as_ptr() as *const u8);
            }
            let entry = Entry::linked();

            let mut create_info = InstanceCreateInfo {
                s_type: InstanceCreateInfo::STRUCTURE_TYPE,
                p_application_info: &app_info as *const ApplicationInfo,
                enabled_extension_count: required_extensions.len() as u32,
                pp_enabled_extension_names: required_extensions.as_ptr() as *const *const i8,
                enabled_layer_count: 0,
                ..Default::default()
            };

            // TODO: Very precarious
            //
            // Arrays might get dropped before `create_instance` reads them from the ptr
            let validation_layers = ["VK_LAYER_KHRONOS_validation\0"];
            let layer_names = &validation_layers.map(|layer| layer.as_ptr() as *const i8);
            if enable_validation {
                create_info.enabled_layer_count = validation_layers.len() as u32;
                create_info.pp_enabled_layer_names = layer_names.as_ptr();
            }

            let instance = entry.create_instance(&create_info, None)?;
            let mut debug_messenger = None;
            let mut debug_messenger_create_func =
                MaybeUninit::<PFN_vkCreateDebugUtilsMessengerEXT>::uninit();
            let mut debug_messenger_destroy_func =
                MaybeUninit::<PFN_vkDestroyDebugUtilsMessengerEXT>::uninit();

            if enable_validation {
                use ash::vk::DebugUtilsMessengerCreateInfoEXT;
                use std::ptr::null;

                let vk_instance = instance.handle();

                let debug_messanger_create_info = DebugUtilsMessengerCreateInfoEXT {
                    s_type: DebugUtilsMessengerCreateInfoEXT::STRUCTURE_TYPE,
                    message_severity: DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                        | DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | DebugUtilsMessageSeverityFlagsEXT::ERROR,
                    message_type: DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                    pfn_user_callback: Some(Self::debug_message_callback),
                    p_user_data: null::<c_void>() as *mut c_void,
                    ..Default::default()
                };

                let create_func_name = c"vkCreateDebugUtilsMessengerEXT";
                let Some(create_func) = entry
                    .get_instance_proc_addr(vk_instance, create_func_name.as_ptr())
                    .map(|func| std::mem::transmute::<_, PFN_vkCreateDebugUtilsMessengerEXT>(func))
                else {
                    eprintln!("Error loading debug messenger create function");
                    return Err(VulkanRendererError::InvalidDebugMessenger);
                };
                debug_messenger_create_func = MaybeUninit::new(create_func);

                let destroy_func_name = c"vkDestroyDebugUtilsMessengerEXT";
                let Some(destroy_func) = entry
                    .get_instance_proc_addr(vk_instance, destroy_func_name.as_ptr())
                    .map(|func| {
                        std::mem::transmute::<_, PFN_vkDestroyDebugUtilsMessengerEXT>(func)
                    })
                else {
                    eprintln!("Error loading debug messenger destroy function");
                    return Err(VulkanRendererError::InvalidDebugMessenger);
                };
                debug_messenger_destroy_func = MaybeUninit::new(destroy_func);

                let mut messenger = std::mem::MaybeUninit::<DebugUtilsMessengerEXT>::uninit();

                let res = (create_func)(
                    vk_instance,
                    &debug_messanger_create_info as *const DebugUtilsMessengerCreateInfoEXT,
                    null(),
                    messenger.as_mut_ptr(),
                );
                if res.result().is_err() {
                    return Err(VulkanRendererError::InvalidDebugMessenger);
                }

                // Should be init by the time we get here
                //
                // Uninit would have returned early in the else above
                debug_messenger = Some(messenger.assume_init());
            }

            Ok(Self {
                _entry: entry,
                instance,
                debug_messenger: debug_messenger.map(|messenger| DebugMessengerData {
                    messenger,
                    create_func: debug_messenger_create_func.assume_init(),
                    destroy_func: debug_messenger_destroy_func.assume_init(),
                }),
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
        if let Some(messenger) = &self.debug_messenger {
            unsafe { (messenger.destroy_func)(self.instance.handle(), messenger.messenger, null()) }
        }
        self.debug_messenger = None;
        unsafe { self.instance.destroy_instance(None) };
    }
}
