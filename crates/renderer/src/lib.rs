pub use window131::Window;

pub trait Renderer {
    /// Connect the renderer to a window
    ///
    /// `window`: Only one renderer can draw to a window at a time, so the renderer will take
    /// ownership of the window for as long as it needs to be able to draw to it
    fn connect_to_window(&mut self, window: &mut Window);
}
