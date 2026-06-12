#[derive(Debug, Error)]
pub enum WindowError {
    #[error("{0}")]
    WindowCreateError(String),

    #[error("{0}")]
    GLFWInitError(#[from] glfw::InitError),
}

#[derive(Debug, Copy, Clone)]
pub enum WindowMode {
    Windowed,
    Borderless,
    BorderlessFullscreen,
    ExclusiveFullscreen,
}

#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub title: String,
    pub size: Vec2u32,
    pub mode: WindowMode,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            title: "Engine131".to_string(),
            size: Vec2u32::default(),
            mode: WindowMode::Windowed,
        }
    }
}

impl WindowSettings {
    pub fn new() -> Self {
        Default::default()
    }
}

impl WindowSettings {
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
    pub fn with_size(mut self, size: Vec2u32) -> Self {
        self.size = size;
        self
    }
    pub fn with_mode(mut self, mode: WindowMode) -> Self {
        self.mode = mode;
        self
    }
}

pub struct Window {
    #[cfg(feature = "GLFW")]
    data: WindowDataGLFW,
    should_close: bool,
}

#[cfg(feature = "GLFW")]
pub mod glfw_manager;
#[cfg(feature = "GLFW")]
pub use glfw_manager::*;
use math131::Vec2u32;
use thiserror::Error;
