use math131::Vec2u32;
use std::any::Any;
use strum::EnumDiscriminants;
use thiserror::Error;
use utils131::define_handle;

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

#[derive(EnumDiscriminants, Debug)]
#[strum_discriminants(derive(Hash))]
#[strum_discriminants(name(WindowEventType))]
pub enum WindowEvent {
    ResizeWindow(Vec2u32),
    ResizeFramebuffer(Vec2u32),
    Close,
}

type CallbackBox = Box<dyn Fn(&dyn Window, WindowEvent)>;
define_handle! {Callback}

pub trait Window
where
    Self: Any,
{
    fn should_close(&self) -> bool;

    fn create_callback_raw(
        &mut self,
        on_event: WindowEventType,
        callback: CallbackBox,
    ) -> CallbackHandle;
    fn remove_callback(&mut self, callback: CallbackHandle);

    fn update(&mut self);
}

impl dyn Window {
    pub fn downcast_ref<T: Window + 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }

    pub fn downcast_mut<T: Window + 'static>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
    }

    pub fn set_callback<C: Fn(&dyn Window, WindowEvent) + 'static>(
        &mut self,
        on_event: WindowEventType,
        callback: C,
    ) -> CallbackHandle {
        self.create_callback_raw(on_event, Box::new(callback))
    }
}

#[cfg(feature = "GLFW")]
pub mod glfw_manager;

#[cfg(feature = "GLFW")]
pub use glfw_manager::*;
