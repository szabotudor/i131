use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub title: String,
    pub size: Vector<u32, 2>,
}

impl Default for WindowSettings {
    fn default() -> Self {
        let size = Vector::<u32, 2>::default();
        Self {
            title: "Engine131".to_string(),
            size,
        }
    }
}

#[cfg(feature = "GLFW")]
struct WindowDataGLFW {}

pub struct Window {
    #[cfg(feature = "GLFW")]
    data: WindowDataGLFW,
}

#[cfg(feature = "GLFW")]
pub mod glfw_manager;
#[cfg(feature = "GLFW")]
pub use glfw_manager::*;

use math131::Vector;
