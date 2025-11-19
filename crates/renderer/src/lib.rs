use anyhow::{Ok, Result};
use backends::BackendInfo;
use window131::Window;

#[cfg(feature = "vulkan")]
use {backends::Backend, vulkan_backend::VulkanBackend};

#[derive(Debug)]
pub struct Renderer {
    backend: Box<dyn Backend>,
}

impl Renderer {
    pub fn backends() -> Vec<BackendInfo> {
        vec![
            #[cfg(feature = "vulkan")]
            VulkanBackend::default().info(),
        ]
    }

    /// Create a new renderer
    ///
    /// `backend`: The rendering backend will be created from the given info.
    /// Use `Renderer::backends()` to get all available backends
    ///
    /// `window`: Only one renderer can draw to a window at a time, so the renderer will take
    /// ownership of the window for as long as it needs to be able to draw to it
    pub fn new(backend: BackendInfo, window: Window) -> Result<Self> {
        Ok(Self { backend: todo!() })
    }
}
