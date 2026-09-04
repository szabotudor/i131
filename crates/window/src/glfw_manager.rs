use glfw::{ClientApiHint, Glfw, GlfwReceiver, PWindow, WindowEvent, WindowHint};
use math131::Vec2u32;

use crate::{Window, WindowError};

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

    pub fn get_glfw_data_mut(&mut self) -> &mut WindowDataGLFW {
        &mut self.data
    }

    pub fn new(settings: super::WindowSettings) -> Result<Self, WindowError> {
        let mut glfw = glfw::init(Self::glfw_error_callback)?;

        glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
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
            resize_callback: None,
        })
    }

    pub fn should_close(&self) -> bool {
        self.should_close || self.data.window.should_close()
    }

    pub fn set_resize_callback<C: Fn(&Window, Vec2u32) + 'static>(&mut self, callback: C) {
        self.resize_callback = Some(Box::new(callback));
    }

    pub fn update(&mut self) {
        self.data.glfw.poll_events();

        for (_e, event) in glfw::flush_messages(&self.data.events) {
            match event {
                WindowEvent::Refresh
                | WindowEvent::Focus(_)
                | WindowEvent::Iconify(_)
                | WindowEvent::MouseButton(_, _, _)
                | WindowEvent::CursorPos(_, _)
                | WindowEvent::CursorEnter(_)
                | WindowEvent::Scroll(_, _)
                | WindowEvent::Key(_, _, _, _)
                | WindowEvent::Char(_)
                | WindowEvent::CharModifiers(_, _)
                | WindowEvent::FileDrop(_)
                | WindowEvent::Maximize(_)
                | WindowEvent::ContentScale(_, _)
                | WindowEvent::Pos(_, _) => {}

                WindowEvent::Size(_, _) => {}
                WindowEvent::FramebufferSize(width, height) => {
                    if let Some(resize_callback) = &self.resize_callback {
                        (resize_callback)(self, dbg!(Vec2u32::new(width as u32, height as u32)));
                    }
                }
                WindowEvent::Close => {
                    self.should_close = true;
                }
            }
        }
    }
}
