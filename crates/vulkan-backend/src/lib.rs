use anyhow::{Ok, Result};
use backends::{Backend, BackendInfo};

#[derive(Debug, Default)]
pub struct VulkanBackend {}

impl VulkanBackend {
    pub fn new() -> Result<Self> {
        Ok(VulkanBackend {})
    }
}

impl Backend for VulkanBackend {
    fn info(self) -> BackendInfo {
        BackendInfo {
            name: "Vulkan".to_string(),
        }
    }

    fn init(&mut self) -> Result<()> {
        todo!()
    }
}
