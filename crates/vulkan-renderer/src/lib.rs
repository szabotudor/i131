use std::os::raw::c_void;

use ash::vk::{
    API_VERSION_1_3, ApplicationInfo, InstanceCreateInfo, TaggedStructure, api_version_variant,
    make_api_version,
};
use renderer131::{OptionRendererError, Renderer, RendererError};
use thiserror::Error;
use window131::Window;

#[derive(Error, Debug)]
pub enum VulkanRendererError {}

#[derive(Debug, Default)]
pub struct VulkanRenderer {}

impl VulkanRenderer {
    pub fn new(
        name: &str,
        app_version: (u32, u32, u32),
        window: &Window,
    ) -> Result<Self, RendererError> {
        let appinfo = ApplicationInfo {
            s_type: ApplicationInfo::STRUCTURE_TYPE,
            p_application_name: name as *const str as *const i8,
            application_version: make_api_version(0, app_version.0, app_version.1, app_version.2),
            p_engine_name: "I131" as *const str as *const i8,
            engine_version: make_api_version(0, app_version.0, app_version.1, app_version.2),
            api_version: API_VERSION_1_3,
            ..Default::default()
        };

        let create_info = {
            let glfw = window.get_glfw_data();
            let required_extensions = glfw
                .glfw
                .get_required_instance_extensions()
                .ok_or_renderer_error(RendererError::InitError(
                    "GLFW returned no extensions".to_string(),
                ))?;

            InstanceCreateInfo {
                s_type: InstanceCreateInfo::STRUCTURE_TYPE,
                p_application_info: &appinfo as *const ApplicationInfo,
                enabled_extension_count: required_extensions.len() as u32,
                pp_enabled_extension_names: required_extensions.as_ptr() as *const *const i8,
                enabled_layer_count: 0,
                ..Default::default()
            }
        };
        todo!()
    }
}

impl Renderer for VulkanRenderer {}
