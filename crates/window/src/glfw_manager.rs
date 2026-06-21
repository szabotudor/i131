use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};

use crate::WindowError;

pub struct WindowDataGLFW {
    pub glfw: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
}
unsafe impl Send for WindowDataGLFW {}

impl super::Window {
    fn glfw_error_callback(error: glfw::Error, message: String) {
        println!("GLFW error:\n{error}\n  - {message}");
    }

    pub fn get_glfw_data(&self) -> &WindowDataGLFW {
        &self.data
    }

    pub fn new(settings: super::WindowSettings) -> Result<Self, WindowError> {
        let mut glfw = glfw::init(Self::glfw_error_callback)?;
        let Some((window, events)) = glfw.create_window(
            settings.size.x,
            settings.size.y,
            settings.title.as_str(),
            glfw::WindowMode::Windowed,
        ) else {
            return Err(WindowError::WindowCreateError(
                "Failed to create GLFW window".to_string(),
            ));
        };

        let data = WindowDataGLFW {
            glfw,
            window,
            events,
        };

        Ok(Self {
            data,
            should_close: false,
        })
    }

    pub fn should_close(&self) -> bool {
        self.should_close || self.data.window.should_close()
    }

    pub fn update(&mut self) {
        self.data.window.make_current();
        self.data.window.swap_buffers();
        self.data.glfw.poll_events();

        for (_e, event) in glfw::flush_messages(&self.data.events) {
            if event == glfw::WindowEvent::Close {
                self.should_close = true;
            }
        }

        glfw::make_context_current(None);
    }
}
