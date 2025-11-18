use anyhow::{Ok, Result};
use backends::BackendInfo;

#[cfg(feature = "vulkan")]
use backends::Backend;
#[cfg(feature = "vulkan")]
use vulkan_backend::VulkanBackend;

pub struct Renderer {}

impl Renderer {
    pub fn backends() -> Vec<BackendInfo> {
        vec![
            #[cfg(feature = "vulkan")]
            VulkanBackend::info(),
        ]
    }

    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}
