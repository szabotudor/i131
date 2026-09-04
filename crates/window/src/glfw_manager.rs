use crate::{
    CallbackBox, CallbackHandle, Window, WindowError, WindowEvent, WindowEventType, WindowSettings,
};
use glfw::{ClientApiHint, Glfw, GlfwReceiver, PWindow, WindowHint};
use math131::Vec2u32;
use std::collections::HashMap;
use utils131::HandleMap;

pub struct WindowGLFW {
    pub glfw: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, glfw::WindowEvent)>,

    callbacks: HashMap<CallbackHandle, WindowEventType>,
    callback_funcs: HashMap<WindowEventType, HandleMap<CallbackHandle, CallbackBox>>,
    should_close: bool,
}

impl WindowGLFW {
    fn glfw_error_callback(error: glfw::Error, message: String) {
        println!("GLFW error:\n{error}\n  - {message}");
    }

    pub fn new(settings: WindowSettings) -> Result<Self, WindowError> {
        let mut glfw = glfw::init(Self::glfw_error_callback)?;

        glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
        let Some((mut window, events)) = glfw.create_window(
            settings.size.x,
            settings.size.y,
            settings.title.as_str(),
            glfw::WindowMode::Windowed,
        ) else {
            return Err(WindowError::WindowCreateError(
                "Failed to create GLFW window".to_string(),
            ));
        };

        window.set_close_polling(true);
        window.set_size_polling(true);
        window.set_framebuffer_size_polling(true);

        Ok(Self {
            glfw,
            window,
            events,

            callbacks: HashMap::default(),
            callback_funcs: HashMap::default(),
            should_close: false,
        })
    }
}

impl Window for WindowGLFW {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn create_callback_raw(
        &mut self,
        on_event: crate::WindowEventType,
        callback: CallbackBox,
    ) -> CallbackHandle {
        let funcs = self.callback_funcs.entry(on_event).or_default();

        let handle = funcs.insert(callback);
        self.callbacks.insert(handle, on_event);

        handle
    }

    fn remove_callback(&mut self, handle: CallbackHandle) {
        if let Some(event_type) = self.callbacks.get(&handle) {
            self.callback_funcs.entry(*event_type).and_modify(|funcs| {
                funcs.remove(handle);
            });
        }

        self.callbacks.remove(&handle);
    }

    fn update(&mut self) {
        self.glfw.poll_events();

        for (_e, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Refresh
                | glfw::WindowEvent::Focus(_)
                | glfw::WindowEvent::Iconify(_)
                | glfw::WindowEvent::MouseButton(_, _, _)
                | glfw::WindowEvent::CursorPos(_, _)
                | glfw::WindowEvent::CursorEnter(_)
                | glfw::WindowEvent::Scroll(_, _)
                | glfw::WindowEvent::Key(_, _, _, _)
                | glfw::WindowEvent::Char(_)
                | glfw::WindowEvent::CharModifiers(_, _)
                | glfw::WindowEvent::FileDrop(_)
                | glfw::WindowEvent::Maximize(_)
                | glfw::WindowEvent::ContentScale(_, _)
                | glfw::WindowEvent::Pos(_, _) => {}

                glfw::WindowEvent::Size(_, _) => {}
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    if let Some(callbacks) =
                        self.callback_funcs.get(&WindowEventType::ResizeFramebuffer)
                    {
                        for (_, callback) in callbacks.iter() {
                            (callback)(
                                self,
                                WindowEvent::ResizeFramebuffer(Vec2u32::new(
                                    width as u32,
                                    height as u32,
                                )),
                            );
                        }
                    }
                }
                glfw::WindowEvent::Close => {
                    self.should_close = true;
                }
            }
        }
    }
}
