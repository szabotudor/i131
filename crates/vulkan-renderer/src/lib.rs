use renderer131::Renderer;
use thiserror::Error;
use window131::Window;

#[derive(Error, Debug)]
pub enum VulkanRendererError {}

#[derive(Debug, Default)]
pub struct VulkanRenderer {}

impl VulkanRenderer {
    pub fn new() -> Result<Self, VulkanRendererError> {
        Ok(VulkanRenderer {})
    }
}

impl Renderer for VulkanRenderer {
    fn connect_to_window(&mut self, window: &mut Window) {
        let _ = window;
        todo!()
    }
}
