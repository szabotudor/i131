use crate::{
    ExecutionContext, ProgramData, VulkanBufferData, VulkanPipelineData, VulkanRenderer,
    VulkanRendererError, VulkanShaderData,
};
use ash::vk::{self, Handle, TaggedStructure};
use renderer131::{
    BufferBinding, BufferCreateInfo, BufferFieldFormat, BufferHandle, BufferUsage,
    ComponentBitCount, DrawCall, DrawMode, ExecutionContextHandle, ProgramHandle, ScalarKind,
    Settings, ShaderCreateInfo, ShaderHandle,
};
use std::{
    collections::HashSet,
    ffi::CString,
    hash::{DefaultHasher, Hash, Hasher},
    ptr::{null, null_mut},
    str::FromStr,
};

macro_rules! vk_format_table {
    (
        uniform {
            $( ($kind:ident, $norm:literal, $width:literal x $count:literal) => $variant:ident; )*
        }
        packed {
            $( ($kind_p:ident, $norm_p:literal, [$($bit:literal),+]) => $variant_p:ident; )*
        }
    ) => {
        fn find_vulkan_format(format: BufferFieldFormat) -> Result<vk::Format, VulkanRendererError> {
            let bits: Vec<u8> = match &format.bits_per_component {
                ComponentBitCount::Scalar(a) => vec![*a],
                ComponentBitCount::Two { a, b } => vec![*a, *b],
                ComponentBitCount::Three { r, g, b } => vec![*r, *g, *b],
                ComponentBitCount::Four { r, g, b, a } => vec![*r, *g, *b, *a],
            };

            // Packed formats are order-sensitive and width-mixed — matched first,
            // as an exact slice pattern per component.
            match (format.kind, format.normalized, bits.as_slice()) {
                $( (ScalarKind::$kind_p, $norm_p, [$($bit),+]) => return Ok(vk::Format::$variant_p), )*
                _ => {}
            }

            // Uniform formats: every component the same width, matched by (kind, norm, width, count).
            if let Some(&width) = bits.first() {
                if bits.iter().all(|&b| b == width) {
                    match (format.kind, format.normalized, width, bits.len()) {
                        $( (ScalarKind::$kind, $norm, $width, $count) => return Ok(vk::Format::$variant), )*
                        _ => {}
                    }
                }
            }

            Err(VulkanRendererError::UnsupportedVertexFormat(format))
        }
    };
}

impl VulkanRenderer {
    fn settings_hash(
        #[expect(
            unused_variables,
            reason = "No setting used in fixed state, only variable state"
        )]
        settings: &Settings,
    ) -> usize {
        #[expect(unused_mut, reason = "Hasher will need to hash settings in the future")]
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

    unsafe fn destroy_pipeline(
        &mut self,
        pipeline_id: usize,
        ignore_buffer: Option<BufferHandle>,
        ignore_program: Option<ProgramHandle>,
    ) -> Result<(), VulkanRendererError> {
        unsafe {
            let pipeline = self.pipelines.get(&pipeline_id).ok_or_else(|| {
                VulkanRendererError::VulkanError(format!(
                    "Pipeline {pipeline_id} doesn't exist, so it cannot be destroyed"
                ))
            })?;

            for program_handle in &pipeline.associated_programs {
                if ignore_program.is_some_and(|program| program == *program_handle) {
                    continue;
                }
                let program = self.programs.get_mut(program_handle).ok_or_else(|| {
                    VulkanRendererError::VulkanError(format!(
                        "Program {program_handle:?} doesn't exist"
                    ))
                })?;

                program.associated_pipelines.remove(&pipeline_id);
            }
            for buffer_handle in &pipeline.associated_buffers {
                if ignore_buffer.is_some_and(|buffer| buffer == *buffer_handle) {
                    continue;
                }
                let buffer = self.buffers.get_mut(*buffer_handle).ok_or_else(|| {
                    VulkanRendererError::VulkanError(format!(
                        "Buffer {buffer_handle:?} doesn't exist"
                    ))
                })?;

                buffer.associated_pipelines.remove(&pipeline_id);
            }

            self.device.destroy_pipeline(pipeline.pipeline, None);
            self.device.destroy_pipeline_layout(pipeline.layout, None);
            self.pipelines.remove(&pipeline_id);

            Ok(())
        }
    }
    unsafe fn get_or_create_pipeline(
        &mut self,
        program: ProgramHandle,
        vertex_buffers: &[BufferHandle],
    ) -> Result<usize, VulkanRendererError> {
        let ProgramData {
            shaders,
            associated_pipelines,
        } = self.programs.get_mut(&program).ok_or_else(|| {
            VulkanRendererError::VulkanError(format!("Program {program:?} doesn't exist"))
        })?;

        let hash = Self::pipeline_hash(shaders, vertex_buffers, &self.settings);

        if self.pipelines.contains_key(&hash) {
            return Ok(hash);
        }

        let shaders = shaders.clone();

        associated_pipelines.insert(hash);
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
            x: self.settings.viewport.origin.x as f32,
            y: self.settings.viewport.origin.y as f32,
            width: self.settings.viewport.size.x as f32,
            height: self.settings.viewport.size.y as f32,
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

            associated_programs: HashSet::default(),
            associated_buffers: HashSet::default(),
        })
    }

    pub(crate) unsafe fn create_execution_context_impl(
        &mut self,
    ) -> Result<ExecutionContextHandle, VulkanRendererError> {
        unsafe {
            let command_pool_create_info = vk::CommandPoolCreateInfo {
                s_type: vk::CommandPoolCreateInfo::STRUCTURE_TYPE,
                queue_family_index: self
                    .queue_family_indices
                    .graphics
                    .ok_or_else(|| VulkanRendererError::MissingQueue("Graphics".to_string()))?,
                flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                ..Default::default()
            };
            let command_pool = self
                .device
                .create_command_pool(&command_pool_create_info, None)?;

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
                s_type: vk::CommandBufferAllocateInfo::STRUCTURE_TYPE,
                command_pool,
                command_buffer_count: 1,
                level: vk::CommandBufferLevel::PRIMARY,
                ..Default::default()
            };
            let command_buffer = self
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)?
                .drain(..)
                .next()
                .ok_or_else(|| {
                    VulkanRendererError::VulkanError(
                        "Expected command buffer allocation to allocate 1 command buffer"
                            .to_string(),
                    )
                })?;

            let semaphore_create_info = vk::SemaphoreCreateInfo {
                s_type: vk::SemaphoreCreateInfo::STRUCTURE_TYPE,
                ..Default::default()
            };

            let image_available_semaphore =
                self.device.create_semaphore(&semaphore_create_info, None)?;
            let render_finished_semaphore =
                self.device.create_semaphore(&semaphore_create_info, None)?;

            let fence_create_info = vk::FenceCreateInfo {
                s_type: vk::FenceCreateInfo::STRUCTURE_TYPE,
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };
            let fence = self.device.create_fence(&fence_create_info, None)?;

            let context = ExecutionContext {
                command_pool,
                command_buffer,

                image_available_semaphore,
                render_finished_semaphore,
                fence,
            };
            let context_handle = self.contexts.insert(context);

            Ok(context_handle)
        }
    }
    pub(crate) unsafe fn destroy_execution_context_impl(
        &mut self,
        context_handle: ExecutionContextHandle,
    ) -> Result<(), VulkanRendererError> {
        unsafe {
            let context = self
                .contexts
                .get(context_handle)
                .ok_or_else(|| VulkanRendererError::NonexistantExecutionContext(context_handle))?;

            self.device
                .wait_for_fences(&[context.fence], true, u64::MAX)?;

            let context = self
                .contexts
                .remove(context_handle)
                .ok_or_else(|| VulkanRendererError::NonexistantExecutionContext(context_handle))?;

            self.device.destroy_fence(context.fence, None);
            self.device
                .destroy_semaphore(context.render_finished_semaphore, None);
            self.device
                .destroy_semaphore(context.image_available_semaphore, None);
            self.device.destroy_command_pool(context.command_pool, None);

            Ok(())
        }
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
        self.programs.insert(
            program,
            ProgramData {
                shaders: shaders.to_vec(),
                associated_pipelines: HashSet::default(),
            },
        );

        Ok(program)
    }
    pub(crate) unsafe fn destroy_program_impl(
        &mut self,
        program: ProgramHandle,
    ) -> Result<(), VulkanRendererError> {
        let associated_pipelines = self
            .programs
            .get(&program)
            .ok_or_else(|| {
                VulkanRendererError::VulkanError(format!("Program {program:?} doesn't exist"))
            })?
            .associated_pipelines
            .clone();

        unsafe {
            for pipeline_id in associated_pipelines {
                self.destroy_pipeline(pipeline_id, None, Some(program))?;
            }
        }

        Ok(())
    }

    vk_format_table! {
        uniform {
            (Float, false, 32 x 1) => R32_SFLOAT;
            (Float, false, 32 x 2) => R32G32_SFLOAT;
            (Float, false, 32 x 3) => R32G32B32_SFLOAT;
            (Float, false, 32 x 4) => R32G32B32A32_SFLOAT;

            (Float, false, 64 x 1) => R64_SFLOAT;
            (Float, false, 64 x 2) => R64G64_SFLOAT;
            (Float, false, 64 x 3) => R64G64B64_SFLOAT;
            (Float, false, 64 x 4) => R64G64B64A64_SFLOAT;

            (UInt, false, 8 x 1) => R8_UINT;
            (UInt, false, 8 x 2) => R8G8_UINT;
            (UInt, false, 8 x 3) => R8G8B8_UINT;
            (UInt, false, 8 x 4) => R8G8B8A8_UINT;
            (UInt, true,  8 x 1) => R8_UNORM;
            (UInt, true,  8 x 2) => R8G8_UNORM;
            (UInt, true,  8 x 3) => R8G8B8_UNORM;
            (UInt, true,  8 x 4) => R8G8B8A8_UNORM;

            (SInt, false, 8 x 1) => R8_SINT;
            (SInt, false, 8 x 2) => R8G8_SINT;
            (SInt, false, 8 x 3) => R8G8B8_SINT;
            (SInt, false, 8 x 4) => R8G8B8A8_SINT;
            (SInt, true,  8 x 1) => R8_SNORM;
            (SInt, true,  8 x 2) => R8G8_SNORM;
            (SInt, true,  8 x 3) => R8G8B8_SNORM;
            (SInt, true,  8 x 4) => R8G8B8A8_SNORM;

            (UInt, false, 16 x 1) => R16_UINT;
            (UInt, false, 16 x 2) => R16G16_UINT;
            (UInt, false, 16 x 3) => R16G16B16_UINT;
            (UInt, false, 16 x 4) => R16G16B16A16_UINT;
            (UInt, true,  16 x 1) => R16_UNORM;
            (UInt, true,  16 x 2) => R16G16_UNORM;
            (UInt, true,  16 x 3) => R16G16B16_UNORM;
            (UInt, true,  16 x 4) => R16G16B16A16_UNORM;

            (SInt, false, 16 x 1) => R16_SINT;
            (SInt, false, 16 x 2) => R16G16_SINT;
            (SInt, false, 16 x 3) => R16G16B16_SINT;
            (SInt, false, 16 x 4) => R16G16B16A16_SINT;
            (SInt, true,  16 x 1) => R16_SNORM;
            (SInt, true,  16 x 2) => R16G16_SNORM;
            (SInt, true,  16 x 3) => R16G16B16_SNORM;
            (SInt, true,  16 x 4) => R16G16B16A16_SNORM;

            (UInt, false, 32 x 1) => R32_UINT;
            (UInt, false, 32 x 2) => R32G32_UINT;
            (UInt, false, 32 x 3) => R32G32B32_UINT;
            (UInt, false, 32 x 4) => R32G32B32A32_UINT;
            (SInt, false, 32 x 1) => R32_SINT;
            (SInt, false, 32 x 2) => R32G32_SINT;
            (SInt, false, 32 x 3) => R32G32B32_SINT;
            (SInt, false, 32 x 4) => R32G32B32A32_SINT;
        }
        packed {
            (UInt, true, [5, 6, 5])     => R5G6B5_UNORM_PACK16;
            (UInt, true, [5, 5, 5, 1])  => R5G5B5A1_UNORM_PACK16;
            (UInt, true, [10, 10, 10, 2]) => A2B10G10R10_UNORM_PACK32;
            (UInt, false, [10, 10, 10, 2]) => A2B10G10R10_UINT_PACK32;
        }
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
            // Find binding requirements, location etc
            //

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
                    let format = Self::find_vulkan_format(field.format)?;

                    let location = match buffer_binding {
                        BufferBinding::Name(name) => {
                            todo!(
                                "Cannot bind buffer {name}. Vulkan cannot search buffers by name."
                            )
                        }
                        BufferBinding::Location(loc) => *loc,
                        // We don't care about binding location, so we just use an absurd location
                        BufferBinding::IndexBufferBinding => u32::MAX as usize,
                    };

                    Ok(vk::VertexInputAttributeDescription {
                        binding: binding as u32,
                        location: location as u32,
                        format,
                        offset: field.offset_in_item as u32,
                    })
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?;

            let memory_properties = self
                .instance
                .get_physical_device_memory_properties(self.physical_device);

            // Create staging buffer and upload buffer data to it
            //

            let staging_buffer_create_info = vk::BufferCreateInfo {
                s_type: vk::BufferCreateInfo::STRUCTURE_TYPE,
                size: std::mem::size_of_val(data.data) as u64,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };

            let staging_buffer = self
                .device
                .create_buffer(&staging_buffer_create_info, None)?;

            let staging_memory_requirements =
                self.device.get_buffer_memory_requirements(staging_buffer);
            let staging_memory_type_index = Self::find_memory_type_index(
                staging_memory_requirements,
                memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            let staging_alloc_info = vk::MemoryAllocateInfo {
                s_type: vk::MemoryAllocateInfo::STRUCTURE_TYPE,
                allocation_size: staging_memory_requirements.size,
                memory_type_index: staging_memory_type_index,
                ..Default::default()
            };

            let staging_device_memory = self.device.allocate_memory(&staging_alloc_info, None)?;
            self.device
                .bind_buffer_memory(staging_buffer, staging_device_memory, 0)?;

            let gpu_data = self.device.map_memory(
                staging_device_memory,
                0,
                staging_buffer_create_info.size,
                vk::MemoryMapFlags::default(),
            )?;
            std::ptr::copy_nonoverlapping(data.data.as_ptr(), gpu_data as *mut u8, data.data.len());
            self.device.unmap_memory(staging_device_memory);

            // Create real buffer and copy staging buffer data into it
            //

            let buffer_create_info = vk::BufferCreateInfo {
                s_type: vk::BufferCreateInfo::STRUCTURE_TYPE,
                size: std::mem::size_of_val(data.data) as u64,
                usage: match data.usage {
                    BufferUsage::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
                    BufferUsage::Index => vk::BufferUsageFlags::INDEX_BUFFER,
                } | vk::BufferUsageFlags::TRANSFER_DST,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };

            let buffer = self.device.create_buffer(&buffer_create_info, None)?;

            let memory_requirements = self.device.get_buffer_memory_requirements(buffer);
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

            // Copy staging buffer to real buffer
            //

            let utility_context = self.contexts.get(self.utility_context).ok_or_else(|| {
                VulkanRendererError::NonexistantExecutionContext(self.utility_context)
            })?;

            self.device
                .wait_for_fences(&[utility_context.fence], true, u64::MAX)?;
            self.device.reset_fences(&[utility_context.fence])?;

            let temp_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::CommandBufferBeginInfo::STRUCTURE_TYPE,
                flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                ..Default::default()
            };

            self.device.reset_command_buffer(
                utility_context.command_buffer,
                vk::CommandBufferResetFlags::default(),
            )?;

            self.device
                .begin_command_buffer(utility_context.command_buffer, &temp_begin_info)?;

            self.device.cmd_copy_buffer(
                utility_context.command_buffer,
                staging_buffer,
                buffer,
                &[vk::BufferCopy {
                    src_offset: 0,
                    dst_offset: 0,
                    size: staging_buffer_create_info.size,
                }],
            );

            self.device
                .end_command_buffer(utility_context.command_buffer)?;

            let temp_submit_info = vk::SubmitInfo {
                s_type: vk::SubmitInfo::STRUCTURE_TYPE,
                command_buffer_count: 1,
                p_command_buffers: &utility_context.command_buffer,
                ..Default::default()
            };

            self.device.queue_submit(
                self.device_queues.graphics.unwrap(),
                &[temp_submit_info],
                utility_context.fence,
            )?;

            self.device
                .wait_for_fences(&[utility_context.fence], true, u64::MAX)?;

            self.device.destroy_buffer(staging_buffer, None);
            self.device.free_memory(staging_device_memory, None);

            // Finally assign to handle

            let buffer_handle = self.buffers.insert(VulkanBufferData {
                usage: data.usage,
                binding_description,
                attribute_descriptions,
                buffer,
                device_memory,
                index_elements: data.item_count,

                associated_pipelines: HashSet::default(),
            });

            Ok(buffer_handle)
        }
    }

    pub(crate) unsafe fn destroy_buffer_impl(
        &mut self,
        buffer_handle: BufferHandle,
    ) -> Result<(), VulkanRendererError> {
        let buffer = self
            .buffers
            .remove(buffer_handle)
            .ok_or_else(|| VulkanRendererError::NonexistantBuffer(buffer_handle))?;

        unsafe {
            for pipeline_id in buffer.associated_pipelines {
                self.destroy_pipeline(pipeline_id, Some(buffer_handle), None)?;
            }

            self.device.destroy_buffer(buffer.buffer, None);
            self.device.free_memory(buffer.device_memory, None);
        }

        Ok(())
    }

    unsafe fn record_draw_command_buffer(
        &self,
        pipeline: &VulkanPipelineData,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
        buffers: &[BufferHandle],
        draw_mode: DrawMode,
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
                    float32: *self.settings.clear_color.as_slice(),
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
                x: self.settings.viewport.origin.x as f32,
                y: self.settings.viewport.origin.y as f32,
                width: self.settings.viewport.size.x as f32,
                height: self.settings.viewport.size.y as f32,
                min_depth: 0.0f32,
                max_depth: 1.0f32,
            };
            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            };
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);

            let mut vertex_buffers = vec![];
            let mut index_buffer = vk::Buffer::null();
            let mut index_buffer_len = 0;
            for buf in buffers
                .iter()
                .map(|buf| {
                    self.buffers
                        .get(*buf)
                        .ok_or_else(|| VulkanRendererError::NonexistantBuffer(*buf))
                })
                .collect::<Result<Vec<_>, VulkanRendererError>>()?
            {
                match buf.usage {
                    BufferUsage::Vertex => vertex_buffers.push(buf.buffer),
                    BufferUsage::Index => {
                        if !index_buffer.is_null() {
                            return Err(VulkanRendererError::TooManyIndexBuffers);
                        }
                        index_buffer = buf.buffer;
                        index_buffer_len = buf.index_elements as u32;
                    }
                }
            }

            let offsets = vec![0u64; vertex_buffers.len()];
            match draw_mode {
                DrawMode::Raw { vertex_count } => {
                    if !index_buffer.is_null() {
                        return Err(VulkanRendererError::IndexBufferNotAllowedInRawMode);
                    }

                    if !vertex_buffers.is_empty() {
                        self.device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &vertex_buffers,
                            &offsets,
                        );
                    }
                    self.device
                        .cmd_draw(command_buffer, vertex_count as u32, 1, 0, 0);
                }
                DrawMode::Indexed => {
                    if index_buffer.is_null() {
                        return Err(VulkanRendererError::NoIndexBuffer);
                    }

                    if !vertex_buffers.is_empty() {
                        self.device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &vertex_buffers,
                            &offsets,
                        );
                    }
                    self.device.cmd_bind_index_buffer(
                        command_buffer,
                        index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    // self.device
                    //     .cmd_draw(command_buffer, index_buffer_len, 1, 0, 0);
                    self.device
                        .cmd_draw_indexed(command_buffer, index_buffer_len, 1, 0, 0, 0);
                }
            }

            self.device.cmd_end_render_pass(command_buffer);
            self.device.end_command_buffer(command_buffer)?;

            Ok(command_buffer)
        }
    }

    pub(crate) unsafe fn execute_draw_impl(
        &mut self,
        program: ProgramHandle,
        buffers: &[BufferHandle],
        draw_mode: DrawMode,
        context: ExecutionContextHandle,
    ) -> Result<(), VulkanRendererError> {
        unsafe {
            let Some(ExecutionContext {
                #[expect(unused_variables, reason = "Not needed here")]
                command_pool,
                command_buffer,
                image_available_semaphore,
                render_finished_semaphore,
                fence,
            }) = self.contexts.get(context).copied()
            else {
                return Err(VulkanRendererError::NonexistantExecutionContext(context));
            };

            self.device.device_wait_idle()?;

            self.device.wait_for_fences(&[fence], true, u64::MAX)?;

            if let Some(new_size) = self.framebuffer_resized {
                self.recreate_swapchain()?;

                if new_size.x == 0 || new_size.y == 0 {
                    return Ok(());
                }

                self.framebuffer_resized = None;
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

            self.device.reset_fences(&[fence])?;

            let pipeline = self.get_or_create_pipeline(program, buffers)?;
            let pipeline = self.pipelines.get(&pipeline).ok_or_else(|| {
                VulkanRendererError::VulkanError(format!(
                    "Pipeline doesn't exist for program {program:?}"
                ))
            })?;

            let command_buffer = self.record_draw_command_buffer(
                pipeline,
                command_buffer,
                image_index as usize,
                buffers,
                draw_mode,
            )?;

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
                fence,
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

            Ok(())
        }
    }

    pub(crate) unsafe fn execute_impl(
        &mut self,
        draw_call: DrawCall,
        context: ExecutionContextHandle,
    ) -> Result<(), VulkanRendererError> {
        match draw_call {
            DrawCall::Draw {
                program,
                buffers,
                mode,
            } => {
                unsafe { self.execute_draw_impl(program, &buffers, mode, context)? };
            }
        }

        Ok(())
    }
}
