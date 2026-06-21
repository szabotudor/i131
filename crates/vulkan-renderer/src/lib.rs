use ash::{
    Entry, Instance, LoadingError,
    vk::{API_VERSION_1_3, ApplicationInfo, InstanceCreateInfo, TaggedStructure, make_api_version},
};
use renderer131::{Renderer, RendererError, RendererInstanceError};
use thiserror::Error;
use window131::WindowDataGLFW;

#[derive(Error, Debug)]
pub enum VulkanRendererError {
    #[cfg(feature = "GLFW")]
    #[error("Error getting GLFW instance")]
    GLFWInstanceError,

    #[error("Error loading vulkan library: {0}")]
    LoadingError(#[from] LoadingError),

    #[error("Vulkan API error: {0}")]
    VulkanAPIError(#[from] ash::vk::Result),
}
impl RendererInstanceError for VulkanRendererError {}

pub struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,
}

impl VulkanRenderer {
    #[cfg(feature = "GLFW")]
    pub fn new_glfw_impl(
        name: &str,
        app_version: (u32, u32, u32),
        window: &WindowDataGLFW,
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
        let required_extensions = required_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let create_info = InstanceCreateInfo {
            s_type: InstanceCreateInfo::STRUCTURE_TYPE,
            p_application_info: &app_info as *const ApplicationInfo,
            enabled_extension_count: required_extensions.len() as u32,
            pp_enabled_extension_names: required_extensions.as_ptr() as *const *const i8,
            enabled_layer_count: 0,
            ..Default::default()
        };

        unsafe {
            let entry = Entry::linked();

            panic!("Debug INFO");
            let validation_layers = &["VK_LAYER_KRONOS_validation"];

            let properties = entry.enumerate_instance_layer_properties()?;

            let validation_layer = properties.iter().find(|layer| {
                let layer_name =
                    str::from_utf8_unchecked(std::mem::transmute(&layer.layer_name as &[i8]));
                validation_layers.contains(&layer_name)
            });

            let instance = entry.create_instance(&create_info, None)?;

            Ok(Self {
                _entry: entry,
                instance,
            })
        }
    }
    #[cfg(feature = "GLFW")]
    pub fn new_glfw(
        name: &str,
        app_version: (u32, u32, u32),
        window: &WindowDataGLFW,
    ) -> Result<Self, RendererError> {
        Ok(Self::new_glfw_impl(name, app_version, window)?)
    }
}

impl Renderer for VulkanRenderer {}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}
