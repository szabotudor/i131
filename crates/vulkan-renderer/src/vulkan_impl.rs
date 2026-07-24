use std::{ffi::CString, ptr::null, str::FromStr};

use ash::vk::{self, TaggedStructure};
use renderer131::{ShaderCreateInfo, ShaderHandle};

use crate::{VulkanRenderer, VulkanRendererError};

impl VulkanRenderer {
    unsafe fn create_pipeline_and_dynamic_state(&mut self) -> Result<(), VulkanRendererError> {
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::PipelineDynamicStateCreateInfo::STRUCTURE_TYPE,
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let vertex_input_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::PipelineVertexInputStateCreateInfo::STRUCTURE_TYPE,
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions: null(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions: null(),
            ..Default::default()
        };

        let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::PipelineInputAssemblyStateCreateInfo::STRUCTURE_TYPE,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width: self.swapchain.extent.width as f32,
            height: self.swapchain.extent.height as f32,
            min_depth: 0.0f32,
            max_depth: 1.0f32,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent,
        };

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::PipelineViewportStateCreateInfo::STRUCTURE_TYPE,
            viewport_count: 1,
            p_viewports: &viewport as *const vk::Viewport,
            scissor_count: 1,
            p_scissors: &scissor as *const vk::Rect2D,
            ..Default::default()
        };

        let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::PipelineRasterizationStateCreateInfo::STRUCTURE_TYPE,
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0f32,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.0f32,
            depth_bias_clamp: 0.0f32,
            depth_bias_slope_factor: 0.0f32,
            ..Default::default()
        };

        let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::PipelineMultisampleStateCreateInfo::STRUCTURE_TYPE,
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0f32,
            p_sample_mask: null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };
        let color_blending_create_info = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::PipelineColorBlendStateCreateInfo::STRUCTURE_TYPE,
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment as *const vk::PipelineColorBlendAttachmentState,
            blend_constants: [0.0f32, 0.0f32, 0.0f32, 0.0f32],
            ..Default::default()
        };

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::PipelineLayoutCreateInfo::STRUCTURE_TYPE,
            set_layout_count: 0,
            p_set_layouts: null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: null(),
            ..Default::default()
        };

        let pipeline_layout = unsafe {
            self.device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
        }?;

        todo!()
    }

    pub(crate) unsafe fn create_shaders_impl(
        &mut self,
        infos: &[ShaderCreateInfo],
    ) -> Result<Vec<ShaderHandle>, VulkanRendererError> {
        unsafe {
            let shader_modules = infos
                .iter()
                .map(|info| {
                    let shader_module_create_info = vk::ShaderModuleCreateInfo {
                        s_type: vk::ShaderModuleCreateInfo::STRUCTURE_TYPE,
                        code_size: info.source.len(),
                        p_code: info.source.as_ptr() as *const u32,
                        ..Default::default()
                    };

                    let shader_module = self
                        .device
                        .create_shader_module(&shader_module_create_info, None)?;
                    Ok((shader_module, CString::from_str(&info.name)?, info))
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?;

            let shaders = shader_modules
                .iter()
                .map(|(shader_module, name, info)| {
                    let shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
                        s_type: vk::PipelineShaderStageCreateInfo::STRUCTURE_TYPE,
                        stage: match info.stage {
                            renderer131::ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                            renderer131::ShaderStage::Pixel => vk::ShaderStageFlags::FRAGMENT,
                            renderer131::ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
                        },
                        module: *shader_module,
                        p_name: name.as_ptr(),
                        ..Default::default()
                    };
                    Ok(shader_stage_create_info)
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?;

            self.create_pipeline_and_dynamic_state()?;
            todo!()
        }
    }

    pub(crate) unsafe fn destroy_shaders_impl(
        &mut self,
        shaders: &[ShaderHandle],
    ) -> Result<(), VulkanRendererError> {
        unsafe {
            self.device.destroy_shader_module(todo!(), None);

            Ok(())
        }
    }
}
