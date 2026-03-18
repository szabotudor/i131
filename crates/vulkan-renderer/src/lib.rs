use anyhow::{Ok, Result};
use renderer131::Renderer;
use window131::Window;

#[derive(Debug, Default)]
pub struct VulkanRenderer {}

impl VulkanRenderer {
    pub fn new() -> Result<Self> {
        Ok(VulkanRenderer {})
    }
}

impl Renderer for VulkanRenderer {
    fn connect_to_window(&mut self, window: &mut Window) {
        todo!()
    }
}
