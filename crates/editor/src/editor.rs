use engine131::{
    math131::Vec2u32,
    renderer131::{
        ProgramHandle, Renderer, RendererError, ShaderCreateInfo, ShaderHandle, ShaderStage,
    },
    systems::{System, SystemId},
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
    window: Window,
    renderer: Box<dyn Renderer>,
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
            window,
            renderer: Box::new(renderer),
            vert: ShaderHandle::null(),
            frag: ShaderHandle::null(),
            prog: ProgramHandle::null(),
        })
    }
}

impl System for Editor {
    fn initialize(
        &mut self,
        engine: &engine131::I131,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;

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

    fn begin_play(
        &mut self,
        engine: &engine131::I131,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;
        Ok(())
    }

    fn update(
        &mut self,
        engine: &engine131::I131,
        delta: f32,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = (engine, delta);

        self.renderer.execute(self.prog)?;

        self.window.update();

        if self.window.should_close() {
            engine.request_immediate_shutdown()?;
        }

        Ok(())
    }

    fn in_editor_update(
        &mut self,
        engine: &engine131::I131,
        delta: f32,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = (engine, delta);

        self.window.update();

        if self.window.should_close() {
            engine.request_immediate_shutdown()?;
        }

        Ok(())
    }

    fn end_play(
        &mut self,
        engine: &engine131::I131,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;
        Ok(())
    }

    fn destroy(&mut self, engine: &engine131::I131) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;

        if !self.vert.is_null() {
            self.renderer.destroy_shader(self.vert)?;
        }
        if !self.frag.is_null() {
            self.renderer.destroy_shader(self.frag)?;
        }

        Ok(())
    }

    fn dependencies() -> &'static [engine131::systems::SystemId]
    where
        Self: Sized,
    {
        &[]
    }

    fn system_id() -> engine131::systems::SystemId
    where
        Self: Sized,
    {
        SystemId("Editor131")
    }
}
