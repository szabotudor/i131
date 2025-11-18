use anyhow::{Ok, Result};
use backends::{Backend, BackendInfo};

pub struct VulkanBackend {}

impl VulkanBackend {
    pub fn new() -> Result<Self> {
        Ok(VulkanBackend {})
    }
}

impl Backend for VulkanBackend {
    fn info() -> BackendInfo {
        BackendInfo {
            name: "Vulkan".to_string(),
        }
    }
}
