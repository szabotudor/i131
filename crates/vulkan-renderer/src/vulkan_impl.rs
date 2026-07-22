use std::{ffi::CString, str::FromStr};

use ash::vk::{self, TaggedStructure};
use renderer131::{ShaderCreateInfo, ShaderHandle};

use crate::{VulkanRenderer, VulkanRendererError};

impl VulkanRenderer {
    pub(crate) unsafe fn create_shader_impl(
        &mut self,
        info: ShaderCreateInfo,
    ) -> Result<ShaderHandle, VulkanRendererError> {
        unsafe {
            let shader_module_create_info = vk::ShaderModuleCreateInfo {
                s_type: vk::ShaderModuleCreateInfo::STRUCTURE_TYPE,
                code_size: info.source.len(),
                p_code: info.source.as_ptr() as *const u32,
                ..Default::default()
            };

            let shader_module = self
                .device
                .create_shader_module(&shader_module_create_info, None)?;

            let p_name = CString::from_str(&info.name)?;
            let shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
                s_type: vk::PipelineShaderStageCreateInfo::STRUCTURE_TYPE,
                stage: match info.stage {
                    renderer131::ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                    renderer131::ShaderStage::Pixel => vk::ShaderStageFlags::FRAGMENT,
                    renderer131::ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
                },
                module: shader_module,
                p_name: p_name.as_ptr(),
                ..Default::default()
            };

            self.shaders.insert(
                ShaderHandle::null(),
                crate::VulkanShaderData { shader_module },
            );
            todo!()
        }
    }

    pub(crate) unsafe fn destroy_shader_impl(
        &mut self,
        shader: ShaderHandle,
    ) -> Result<(), VulkanRendererError> {
        unsafe {
            self.device.destroy_shader_module(todo!(), None);

            Ok(())
        }
    }
}
