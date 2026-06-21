use engine131::{
    math131::Vec2u32,
    renderer131::{Renderer, RendererError},
    systems::{System, SystemId},
    window131::{Window, WindowError, WindowMode, WindowSettings},
};
use thiserror::Error;
use vulkan_renderer::VulkanRenderer;

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
}
impl Editor {
    pub fn new() -> Result<Self, EditorError> {
        let window = Window::new(
            WindowSettings::new()
                .with_title("I131".to_string())
                .with_size(Vec2u32::new(800, 600))
                .with_mode(WindowMode::Windowed),
        )?;
        let renderer =
            VulkanRenderer::new_glfw("I131_VulkanBackend", (1, 0, 0), window.get_glfw_data())?;

        Ok(Self {
            window,
            renderer: Box::new(renderer),
        })
    }
}

impl System for Editor {
    fn initialize(
        &mut self,
        engine: &engine131::I131,
    ) -> Result<(), engine131::systems::SystemError> {
        let _ = engine;
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
        self.window.update();
        if self.window.should_close() {
            engine.destroy_system(Self::system_id())?;
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
            engine.destroy_system(Self::system_id())?;
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
