use crate::{
    FlowControl, MAX_FRAMES_IN_FLIGHT, VulkanBufferData, VulkanPipelineData, VulkanRenderer,
    VulkanRendererError, VulkanShaderData,
};
use ash::vk::{self, TaggedStructure};
use renderer131::{
    BufferCreateInfo, BufferFieldFormat, BufferHandle, BufferUsage, ComponentBitCount, DrawCall,
    ProgramHandle, ScalarKind, Settings, ShaderCreateInfo, ShaderHandle,
};
use std::{
    ffi::CString,
    hash::{DefaultHasher, Hash, Hasher},
    ptr::{null, null_mut},
    str::FromStr,
};

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
    fn vertex_buffers_hash(vertex_buffers: &[BufferHandle]) -> usize {
        let mut hasher = DefaultHasher::new();
        for buffer in vertex_buffers {
            buffer.hash(&mut hasher);
        }
        let hash = hasher.finish();

        hash as usize
    }
    fn pipeline_hash(
        shaders: &[ShaderHandle],
        vertex_buffers: &[BufferHandle],
        settings: &Settings,
    ) -> usize {
        let mut hasher = DefaultHasher::new();

        let settings_hash = Self::settings_hash(settings);
        settings_hash.hash(&mut hasher);

        let shader_hash = Self::shaders_hash(shaders);
        shader_hash.hash(&mut hasher);

        let vertex_buffers_hash = Self::vertex_buffers_hash(vertex_buffers);
        vertex_buffers_hash.hash(&mut hasher);

        let hash = hasher.finish();

        hash as usize
    }

    unsafe fn get_or_create_pipeline(
        &mut self,
        program: ProgramHandle,
        vertex_buffers: &[BufferHandle],
    ) -> Result<usize, VulkanRendererError> {
        let (shaders, pipelines) = self.programs.get_mut(&program).ok_or_else(|| {
            VulkanRendererError::VulkanError(format!("Program {program:?} doesn't exist"))
        })?;

        let hash = Self::pipeline_hash(shaders, vertex_buffers, &self.settings);

        if self.pipelines.contains_key(&hash) {
            return Ok(hash);
        }

        let shaders = shaders.clone();

        pipelines.push(hash);
        let pipeline = unsafe { self.create_pipeline_and_dynamic_state(&shaders, vertex_buffers)? };
        self.pipelines.insert(hash, pipeline);

        Ok(hash)
    }

    unsafe fn create_pipeline_and_dynamic_state(
        &mut self,
        shaders: &[ShaderHandle],
        vertex_buffers: &[BufferHandle],
    ) -> Result<VulkanPipelineData, VulkanRendererError> {
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::PipelineDynamicStateCreateInfo::STRUCTURE_TYPE,
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let (vertex_buffer_bindings, vertex_buffer_attributes) = vertex_buffers
            .iter()
            .filter_map(|buffer| {
                let buffer = self.buffers.get(*buffer)?;

                if buffer.usage != BufferUsage::Vertex {
                    return None;
                }

                Some((
                    buffer.binding_description,
                    buffer.attribute_descriptions.clone(),
                ))
            })
            .fold(
                (Vec::default(), Vec::default()),
                |(mut acc_bind, mut acc_attr), (bind, attr)| {
                    acc_bind.push(bind);
                    acc_attr.extend_from_slice(&attr);
                    (acc_bind, acc_attr)
                },
            );

        let vertex_input_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::PipelineVertexInputStateCreateInfo::STRUCTURE_TYPE,
            vertex_binding_description_count: vertex_buffer_bindings.len() as u32,
            p_vertex_binding_descriptions: vertex_buffer_bindings.as_ptr(),
            vertex_attribute_description_count: vertex_buffer_attributes.len() as u32,
            p_vertex_attribute_descriptions: vertex_buffer_attributes.as_ptr(),
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
                    .get(*shader)
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

        let render_pass = self.render_pass;
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

                    let shader_handle = self.shaders.insert(VulkanShaderData {
                        shader_module,
                        stage: info.stage,
                        name: CString::from_str(&info.name)?,
                    });

                    Ok(shader_handle)
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
                    self.shaders
                        .get(*handle)
                        .ok_or_else(|| VulkanRendererError::NonexistantShader(*handle))
                })
                .collect::<Result<Vec<&VulkanShaderData>, VulkanRendererError>>()?;

            for shader in shader_metas {
                self.device
                    .destroy_shader_module(shader.shader_module, None);
            }

            for shader in shaders {
                self.shaders.remove(*shader);
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

    fn find_memory_type_index(
        memory_requirements: vk::MemoryRequirements,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<u32, VulkanRendererError> {
        for i in 0..memory_properties.memory_type_count {
            if (memory_requirements.memory_type_bits & (1 << i)) != 0
                && (memory_properties.memory_types[i as usize].property_flags & property_flags)
                    .as_raw()
                    != 0
            {
                return Ok(i);
            }
        }

        Err(VulkanRendererError::NoSupportedMemoryLayouts)
    }
    pub(crate) unsafe fn create_buffer_impl(
        &mut self,
        data: BufferCreateInfo,
    ) -> Result<BufferHandle, VulkanRendererError> {
        unsafe {
            let binding = if let Some(binding) = self.freed_buffer_bindings.pop_front() {
                binding
            } else {
                self.buffer_bindings += 1;
                self.buffer_bindings - 1
            };

            let binding_description = vk::VertexInputBindingDescription {
                binding: binding as u32,
                stride: data.item_stride as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            };

            let attribute_descriptions = data
                .item_fields
                .iter()
                .map(|(buffer_binding, field)| {
                    let format = match field.format {
                        BufferFieldFormat {
                            kind: ScalarKind::Float,
                            normalized: false,
                            bits_per_component: ComponentBitCount::Two { a: 32, b: 32 },
                        } => Ok(vk::Format::R32G32_SFLOAT),
                        BufferFieldFormat {
                            kind: ScalarKind::Float,
                            normalized: false,
                            bits_per_component:
                                ComponentBitCount::Three {
                                    r: 32,
                                    g: 32,
                                    b: 32,
                                },
                        } => Ok(vk::Format::R32G32B32_SFLOAT),

                        _ => Err(VulkanRendererError::UnsupportedVertexFormat(
                            field.format.clone(),
                        )),
                    }?;

                    let location = match buffer_binding {
                        renderer131::BufferBinding::Name(name) => todo!("User wants to bind buffer {name} by name.\n Buffer name search in SPIR-V not supported yet"),
                        renderer131::BufferBinding::Location(loc) => *loc,
                    };

                    Ok(vk::VertexInputAttributeDescription {
                        binding: binding as u32,
                        location: location as u32,
                        format,
                        offset: field.offset_in_item as u32,
                    })
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?;

            let buffer_create_info = vk::BufferCreateInfo {
                s_type: vk::BufferCreateInfo::STRUCTURE_TYPE,
                size: std::mem::size_of_val(data.data) as u64,
                usage: vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };

            let buffer = self.device.create_buffer(&buffer_create_info, None)?;

            let memory_requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory_properties = self
                .instance
                .get_physical_device_memory_properties(self.physical_device);

            let memory_type_index = Self::find_memory_type_index(
                memory_requirements,
                memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            let alloc_info = vk::MemoryAllocateInfo {
                s_type: vk::MemoryAllocateInfo::STRUCTURE_TYPE,
                allocation_size: memory_requirements.size,
                memory_type_index,
                ..Default::default()
            };

            let device_memory = self.device.allocate_memory(&alloc_info, None)?;
            self.device.bind_buffer_memory(buffer, device_memory, 0)?;

            let gpu_data = self.device.map_memory(
                device_memory,
                0,
                buffer_create_info.size,
                vk::MemoryMapFlags::default(),
            )?;
            std::ptr::copy_nonoverlapping(data.data.as_ptr(), gpu_data as *mut u8, data.data.len());
            self.device.unmap_memory(device_memory);

            let buffer_handle = self.buffers.insert(VulkanBufferData {
                usage: data.usage,
                binding_description,
                attribute_descriptions,
                buffer,
                device_memory,
            });

            Ok(buffer_handle)
        }
    }

    pub(crate) unsafe fn destroy_buffer_impl(
        &mut self,
        buffer: BufferHandle,
    ) -> Result<(), VulkanRendererError> {
        let buffer = self
            .buffers
            .remove(buffer)
            .ok_or_else(|| VulkanRendererError::NonexistantBuffer(buffer))?;

        unsafe {
            self.device.destroy_buffer(buffer.buffer, None);
            self.device.free_memory(buffer.device_memory, None);
        }

        Ok(())
    }

    unsafe fn record_command_buffer(
        &self,
        pipeline: &VulkanPipelineData,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
    ) -> Result<vk::CommandBuffer, VulkanRendererError> {
        unsafe {
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
                framebuffer: self.swapchain.framebuffers[image_index],
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

            let buffers = self
                .buffers
                .iter()
                .map(|(_, buffer)| buffer.buffer)
                .collect::<Vec<_>>();
            let offsets = vec![0u64; buffers.len()];
            self.device
                .cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);

            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            // TODO: Draw commands

            self.device.cmd_end_render_pass(command_buffer);
            self.device.end_command_buffer(command_buffer)?;

            Ok(command_buffer)
        }
    }

    pub(crate) unsafe fn execute_draw_impl(
        &mut self,
        program: ProgramHandle,
        vertex_buffers: &[BufferHandle],
    ) -> Result<(), VulkanRendererError> {
        // TODO: Should find a better way to recreate swapchain that doesn't force the user into
        // specific code flow
        self.window
            .try_borrow()
            .map_err(|_| VulkanRendererError::WindowAlreadyBorrowedError)?;

        unsafe {
            let command_buffers = self
                .command_buffers
                .get(&self.command_pools.graphics)
                .ok_or_else(|| {
                    VulkanRendererError::VulkanError(
                        "Expected graphics command pool to exist".to_string(),
                    )
                })?;
            let command_buffer = command_buffers[self.current_frame];

            let FlowControl {
                image_available_semaphore,
                render_finished_semaphore,
                in_flight_fence,
            } = *self.flow_control.get(&command_buffer).ok_or_else(|| {
                VulkanRendererError::VulkanError(
                    "Flow control doesn't exist for expected command buffer".to_string(),
                )
            })?;

            self.device.device_wait_idle()?;

            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;

            let framebuffer_new_size = *self.framebuffer_resized.borrow();
            if let Some(new_size) = framebuffer_new_size {
                self.recreate_swapchain()?;

                if new_size.x == 0 || new_size.y == 0 {
                    return Ok(());
                }

                *self.framebuffer_resized.borrow_mut() = None;
            }

            let mut image_index = 0u32;
            match (self.instance_extensions.acquire_next_image_khr)(
                self.device.handle(),
                self.swapchain.swapchain,
                u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
                &mut image_index as *mut u32,
            )
            .result()
            {
                Ok(_) => {}
                Err(err) => match err {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                        self.recreate_swapchain()?;
                    }
                    _ => {}
                },
            }

            self.device.reset_fences(&[in_flight_fence])?;

            let pipeline = self.get_or_create_pipeline(program, vertex_buffers)?;
            let pipeline = self.pipelines.get(&pipeline).ok_or_else(|| {
                VulkanRendererError::VulkanError(format!(
                    "Pipeline doesn't exist for program {program:?}"
                ))
            })?;

            let command_buffer =
                self.record_command_buffer(pipeline, command_buffer, image_index as usize)?;

            let wait_semaphores = [image_available_semaphore];
            let signal_semaphores = [render_finished_semaphore];
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
                in_flight_fence,
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

            match (self.instance_extensions.queue_present_khr)(
                self.device_queues.present.unwrap(),
                &present_info as *const vk::PresentInfoKHR,
            )
            .result()
            {
                Ok(_) => {}
                Err(err) => match err {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                        self.recreate_swapchain()?;
                    }
                    _ => {}
                },
            }

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT as usize;

            Ok(())
        }
    }

    pub(crate) unsafe fn execute_impl(
        &mut self,
        draw_call: DrawCall,
    ) -> Result<(), VulkanRendererError> {
        match draw_call {
            DrawCall::Draw {
                program,
                vertex_buffers,
            } => {
                unsafe { self.execute_draw_impl(program, &vertex_buffers)? };
            }
        }

        Ok(())
    }
}
