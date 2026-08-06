use engine131::{
    math131::Vec2u32,
    renderer131::{
        ProgramHandle, Renderer, RendererError, ShaderCreateInfo, ShaderHandle, ShaderStage,
    },
    systems::{System, SystemContext, SystemId},
    window131::{Window, WindowError, WindowMode, WindowSettings},
};
use thiserror::Error;
use vulkan_renderer::{ValidationLevel, VulkanRenderer};

use crate::shaders_vulkan;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("Window error: {0}")]
    WindowError(#[from] WindowError),

    #[error("Window error: {0}")]
    RendererError(#[from] RendererError),
}

pub(crate) struct Editor {
    context: SystemContext,
    renderer: Box<dyn Renderer>,
    window: Window,
    vert: ShaderHandle,
    frag: ShaderHandle,
    prog: ProgramHandle,
}
unsafe impl Send for Editor {}
unsafe impl Sync for Editor {}

impl Editor {
    pub fn new() -> Result<Self, EditorError> {
        let window = Window::new(
            WindowSettings::new()
                .with_title("I131".to_string())
                .with_size(Vec2u32::new(800, 600))
                .with_mode(WindowMode::Windowed),
        )?;
        // TODO: Implement system init arguments
        //
        // Needed to enable/disable validation here
        let renderer = VulkanRenderer::new_glfw(
            "I131_VulkanBackend",
            (1, 3, 0),
            window.get_glfw_data(),
            ValidationLevel::Normal,
        )?;

        Ok(Self {
            context: SystemContext::empty(),
            window,
            renderer: Box::new(renderer),
            vert: ShaderHandle::null(),
            frag: ShaderHandle::null(),
            prog: ProgramHandle::null(),
        })
    }
}

impl System for Editor {
    const SYSTEM_ID: SystemId = SystemId("Editor");
    const DEPENDENCIES: &'static [SystemId] = &[];
    const BEFORE: &'static [SystemId] = &[];
    const AFTER: &'static [SystemId] = &[];

    fn initialize(
        &mut self,
        context: SystemContext,
    ) -> Result<(), engine131::systems::SystemError> {
        self.context = context;

        let default_vert = self.renderer.create_shader(ShaderCreateInfo {
            source: shaders_vulkan::DEFAULT_VERT,
            stage: ShaderStage::Vertex,
            name: "default.vert".to_string(),
        })?;
        let default_frag = self.renderer.create_shader(ShaderCreateInfo {
            source: shaders_vulkan::DEFAULT_FRAG,
            stage: ShaderStage::Pixel,
            name: "default.frag".to_string(),
        })?;

        self.vert = default_vert;
        self.frag = default_frag;

        let default_prog = self
            .renderer
            .create_program(&[default_vert, default_frag])?;

        self.prog = default_prog;

        Ok(())
    }

    fn begin_play(&mut self) -> Result<(), engine131::systems::SystemError> {
        Ok(())
    }

    fn update(&mut self, delta: f32) -> Result<(), engine131::systems::SystemError> {
        let _ = delta;
        let engine = self.context.engine()?;

        self.renderer.execute(self.prog)?;

        self.window.update();

        if self.window.should_close() {
            engine.request_immediate_shutdown()?;
        }

        Ok(())
    }

    fn in_editor_update(&mut self, delta: f32) -> Result<(), engine131::systems::SystemError> {
        let _ = delta;
        let engine = self.context.engine()?;

        self.window.update();

        if self.window.should_close() {
            engine.request_immediate_shutdown()?;
        }

        Ok(())
    }

    fn end_play(&mut self) -> Result<(), engine131::systems::SystemError> {
        Ok(())
    }

    fn destroy(&mut self) -> Result<(), engine131::systems::SystemError> {
        self.renderer.destroy()?;
        self.vert = ShaderHandle::null();
        self.frag = ShaderHandle::null();
        self.prog = ProgramHandle::null();

        Ok(())
    }
}
