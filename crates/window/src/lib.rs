use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WindowSettings {
    pub title: String,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            title: "Engine131".to_string(),
        }
    }
}

#[cfg(feature = "GLFW")]
struct WindowDataGLFW {}

pub struct Window {
    #[cfg(feature = "GLFW")]
    data: WindowDataGLFW,
}

impl Window {
    #[cfg(feature = "GLFW")]
    pub fn new(settings: WindowSettings) -> Result<Self> {
        todo!()
    }
}
