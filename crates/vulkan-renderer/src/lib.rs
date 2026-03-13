use anyhow::{Ok, Result};

#[derive(Debug, Default)]
pub struct VulkanRenderer {}

impl VulkanRenderer {
    pub fn new() -> Result<Self> {
        Ok(VulkanRenderer {})
    }
}

