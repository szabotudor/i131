use std::{
    ffi::CString,
    hash::{DefaultHasher, Hash, Hasher},
    ptr::{null, null_mut},
    str::FromStr,
};

use ash::vk::{self, TaggedStructure};
use renderer131::{ProgramHandle, Settings, ShaderCreateInfo, ShaderHandle};

use crate::{VulkanPipelineData, VulkanRenderer, VulkanRendererError, VulkanShaderData};

impl VulkanRenderer {
    fn settings_hash(settings: &Settings) -> usize {
        let mut hasher = DefaultHasher::new();
        let hash = hasher.finish();

        hash as usize
    }
    fn shaders_hash(shaders: &[ShaderHandle]) -> usize {
        let mut hasher = DefaultHasher::new();
        for shader in shaders {
            shader.hash(&mut hasher);
        }
        let hash = hasher.finish();

        hash as usize
    }
    fn pipeline_hash(shaders: &[ShaderHandle], settings: &Settings) -> usize {
        let mut hasher = DefaultHasher::new();

        let shader_hash = Self::shaders_hash(shaders);
        shader_hash.hash(&mut hasher);
        Self::settings_hash(settings).hash(&mut hasher);

        let hash = hasher.finish();

        hash as usize
    }

    unsafe fn get_or_create_pipeline(
        &mut self,
        program: ProgramHandle,
    ) -> Result<usize, VulkanRendererError> {
        let (shaders, pipelines) = self.programs.get_mut(&program).ok_or_else(|| {
            VulkanRendererError::VulkanError(format!("Program {program:?} doesn't exist"))
        })?;

        let hash = Self::pipeline_hash(shaders, &self.settings);

        if self.pipelines.contains_key(&hash) {
            return Ok(hash);
        }

        let shaders = shaders.clone();

        pipelines.push(hash);
        let pipeline = unsafe { self.create_pipeline_and_dynamic_state(&shaders)? };
        self.pipelines.insert(hash, pipeline);

        Ok(hash)
    }

    unsafe fn create_pipeline_and_dynamic_state(
        &mut self,
        shaders: &[ShaderHandle],
    ) -> Result<VulkanPipelineData, VulkanRendererError> {
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

        let shader_stages = shaders
            .iter()
            .map(|shader| {
                let info = self
                    .shaders
                    .get(shader)
                    .ok_or(VulkanRendererError::VulkanError(format!(
                        "Shader {shader:?} doesn't exist"
                    )))?;
                let shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
                    s_type: vk::PipelineShaderStageCreateInfo::STRUCTURE_TYPE,
                    stage: match info.stage {
                        renderer131::ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                        renderer131::ShaderStage::Pixel => vk::ShaderStageFlags::FRAGMENT,
                        renderer131::ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
                    },
                    module: info.shader_module,
                    p_name: c"main".as_ptr(),
                    ..Default::default()
                };

                Ok(shader_stage_create_info)
            })
            .collect::<Result<Vec<_>, VulkanRendererError>>()?;

        let render_pass = self.get_or_create_render_pass()?;
        // Create pipeline after pipeline layout
        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::GraphicsPipelineCreateInfo::STRUCTURE_TYPE,
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),

            p_vertex_input_state: &vertex_input_create_info
                as *const vk::PipelineVertexInputStateCreateInfo,
            p_input_assembly_state: &input_assembly_create_info
                as *const vk::PipelineInputAssemblyStateCreateInfo,
            p_viewport_state: &viewport_state_create_info
                as *const vk::PipelineViewportStateCreateInfo,
            p_rasterization_state: &rasterizer_create_info
                as *const vk::PipelineRasterizationStateCreateInfo,
            p_multisample_state: &multisampling_create_info
                as *const vk::PipelineMultisampleStateCreateInfo,
            p_depth_stencil_state: null(),
            p_color_blend_state: &color_blending_create_info
                as *const vk::PipelineColorBlendStateCreateInfo,
            p_dynamic_state: &dynamic_state_create_info
                as *const vk::PipelineDynamicStateCreateInfo,
            layout: pipeline_layout,
            render_pass,
            // Which subpass of the render pass to use
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            ..Default::default()
        };

        let pipeline = *unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .map_err(|(_, err)| err)?
        }
        .first()
        .unwrap();

        Ok(VulkanPipelineData {
            pipeline,
            layout: pipeline_layout,
        })
    }

    fn get_or_create_render_pass(&mut self) -> Result<vk::RenderPass, VulkanRendererError> {
        if self.render_pass != vk::RenderPass::null() {
            return Ok(self.render_pass);
        }

        let color_attachment = vk::AttachmentDescription {
            format: self.swapchain.format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref as *const vk::AttachmentReference,
            ..Default::default()
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::NONE,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::RenderPassCreateInfo::STRUCTURE_TYPE,
            attachment_count: 1,
            p_attachments: &color_attachment as *const vk::AttachmentDescription,
            subpass_count: 1,
            p_subpasses: &subpass as *const vk::SubpassDescription,
            dependency_count: 1,
            p_dependencies: &dependency as *const vk::SubpassDependency,
            ..Default::default()
        };

        let render_pass = unsafe {
            self.device
                .create_render_pass(&render_pass_create_info, None)
        }?;

        let swapchain_framebuffers = self
            .swapchain
            .swapchain_image_views
            .iter()
            .map(|image_view| {
                let attachments = [*image_view];

                let framebuffer_create_info = vk::FramebufferCreateInfo {
                    s_type: vk::FramebufferCreateInfo::STRUCTURE_TYPE,
                    render_pass,
                    attachment_count: 1,
                    p_attachments: attachments.as_ptr(),
                    width: self.swapchain.extent.width,
                    height: self.swapchain.extent.height,
                    layers: 1,
                    ..Default::default()
                };

                let framebuffer = unsafe {
                    self.device
                        .create_framebuffer(&framebuffer_create_info, None)
                }?;
                Ok(framebuffer)
            })
            .collect::<Result<Vec<_>, VulkanRendererError>>()?;

        self.render_pass = render_pass;
        self.swapchain_framebuffers = swapchain_framebuffers;

        Ok(render_pass)
    }

    pub(crate) unsafe fn create_shaders_impl(
        &mut self,
        infos: &[ShaderCreateInfo],
    ) -> Result<Vec<ShaderHandle>, VulkanRendererError> {
        unsafe {
            let shader_handles = infos
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

                    let shader_handle = self.shader_handles;
                    self.shaders.insert(
                        ShaderHandle::from_raw(shader_handle),
                        VulkanShaderData {
                            shader_module,
                            stage: info.stage,
                            name: CString::from_str(&info.name)?,
                        },
                    );
                    self.shader_handles += 1;

                    Ok(ShaderHandle::from_raw(shader_handle))
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?;

            Ok(shader_handles)
        }
    }

    pub(crate) unsafe fn destroy_shaders_impl(
        &mut self,
        shaders: &[ShaderHandle],
    ) -> Result<(), VulkanRendererError> {
        unsafe {
            let shader_metas = shaders
                .iter()
                .map(|handle| {
                    self.shaders.get(handle).ok_or_else(|| {
                        VulkanRendererError::VulkanError(format!(
                            "Shader {handle:?} doesn't exist."
                        ))
                    })
                })
                .collect::<Result<Vec<&VulkanShaderData>, VulkanRendererError>>()?;

            for shader in shader_metas {
                self.device
                    .destroy_shader_module(shader.shader_module, None);
            }

            for shader in shaders {
                self.shaders.remove(shader);
            }

            Ok(())
        }
    }

    pub(crate) unsafe fn create_program_impl(
        &mut self,
        shaders: &[ShaderHandle],
    ) -> Result<ProgramHandle, VulkanRendererError> {
        let program = ProgramHandle::from_raw(Self::shaders_hash(shaders));
        if self.programs.contains_key(&program) {
            return Err(VulkanRendererError::VulkanError(format!(
                "Program {program:?} already exists"
            )));
        }
        self.programs
            .insert(program, (shaders.to_vec(), Vec::default()));

        Ok(program)
    }
    pub(crate) unsafe fn destroy_program_impl(
        &mut self,
        program: ProgramHandle,
    ) -> Result<(), VulkanRendererError> {
        let (_shaders, pipelines) = self.programs.get(&program).ok_or_else(|| {
            VulkanRendererError::VulkanError(format!("Program {program:?} doesn't exist"))
        })?;

        unsafe {
            for pipeline_id in pipelines {
                let pipeline = self.pipelines.get(pipeline_id).ok_or_else(|| {
                    VulkanRendererError::VulkanError(format!(
                        "Pipeline {pipeline_id}, referenced by program {program:?} doesn't exist"
                    ))
                })?;

                self.device.destroy_pipeline(pipeline.pipeline, None);
                self.device.destroy_pipeline_layout(pipeline.layout, None);
                self.pipelines.remove(pipeline_id);
            }
        }

        Ok(())
    }

    unsafe fn record_command_buffer(
        &mut self,
        program: ProgramHandle,
        image_index: usize,
    ) -> Result<vk::CommandBuffer, VulkanRendererError> {
        unsafe {
            let pipeline = self.get_or_create_pipeline(program)?;
            let pipeline = self.pipelines.get(&pipeline).ok_or_else(|| {
                VulkanRendererError::VulkanError(format!(
                    "Pipeline doesn't exist for program {program:?}"
                ))
            })?;

            let command_buffers = self
                .command_buffers
                .get(&self.command_pools.graphics)
                .ok_or_else(|| {
                    VulkanRendererError::VulkanError(
                        "Graphics command pool doesn't have any command buffers".to_string(),
                    )
                })?;
            let command_buffer = *command_buffers.first().ok_or_else(|| {
                VulkanRendererError::VulkanError(
                    "Expected command pool to have at least one command buffer".to_string(),
                )
            })?;

            self.device.reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?;

            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::CommandBufferBeginInfo::STRUCTURE_TYPE,
                flags: vk::CommandBufferUsageFlags::from_raw(0),
                p_inheritance_info: null(),
                ..Default::default()
            };
            self.device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)?;

            // TODO: Image index
            let clear_value = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [
                        self.settings.clear_color.x,
                        self.settings.clear_color.y,
                        self.settings.clear_color.z,
                        self.settings.clear_color.w,
                    ],
                },
            };
            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::RenderPassBeginInfo::STRUCTURE_TYPE,
                render_pass: self.render_pass,
                framebuffer: self.swapchain_framebuffers[image_index],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                },
                clear_value_count: 1,
                p_clear_values: &clear_value as *const vk::ClearValue,
                ..Default::default()
            };

            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );

            let viewport = vk::Viewport {
                x: 0.0f32,
                y: 0.0f32,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                min_depth: 0.0f32,
                max_depth: 0.0f32,
            };
            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            };
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);

            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            // TODO: Draw commands

            self.device.cmd_end_render_pass(command_buffer);
            self.device.end_command_buffer(command_buffer)?;

            Ok(command_buffer)
        }
    }

    pub(crate) unsafe fn execute_impl(
        &mut self,
        program: ProgramHandle,
    ) -> Result<(), VulkanRendererError> {
        unsafe {
            self.device.device_wait_idle()?;

            self.device
                .wait_for_fences(&[self.in_flight_fence], true, u64::MAX)?;
            self.device.reset_fences(&[self.in_flight_fence])?;

            let mut image_index = 0u32;
            (self.instance_extensions.acquire_next_image_khr)(
                self.device.handle(),
                self.swapchain.swapchain,
                u64::MAX,
                self.image_available_semaphore,
                vk::Fence::null(),
                &mut image_index as *mut u32,
            )
            .result()?;

            let command_buffer = self.record_command_buffer(program, image_index as usize)?;

            let wait_semaphores = [self.image_available_semaphore];
            let signal_semaphores = [self.render_finished_semaphore];
            let wait_stage = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

            let submit_info = vk::SubmitInfo {
                s_type: vk::SubmitInfo::STRUCTURE_TYPE,
                wait_semaphore_count: 1,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_stage.as_ptr(),
                command_buffer_count: 1,
                p_command_buffers: &command_buffer as *const vk::CommandBuffer,
                signal_semaphore_count: 1,
                p_signal_semaphores: signal_semaphores.as_ptr(),
                ..Default::default()
            };

            self.device.queue_submit(
                // By now this should already be verified to exist
                self.device_queues.graphics.unwrap(),
                &[submit_info],
                self.in_flight_fence,
            )?;

            let present_info = vk::PresentInfoKHR {
                s_type: vk::PresentInfoKHR::STRUCTURE_TYPE,
                wait_semaphore_count: 1,
                p_wait_semaphores: signal_semaphores.as_ptr(),
                swapchain_count: 1,
                p_swapchains: &self.swapchain.swapchain as *const vk::SwapchainKHR,
                p_image_indices: &image_index as *const u32,
                p_results: null_mut(),
                ..Default::default()
            };

            (self.instance_extensions.queue_present_khr)(
                self.device_queues.present.unwrap(),
                &present_info as *const vk::PresentInfoKHR,
            )
            .result()?;

            Ok(())
        }
    }
}
