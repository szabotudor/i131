use anyhow::Result;

impl super::Window {
    fn glfw_error_callback(error: glfw::Error, message: String) {
        println!("GLFW error:\n{error}\n  - {message}");
    }

    pub fn new(settings: super::WindowSettings) -> Result<Self> {
        let mut glfw = glfw::init(Self::glfw_error_callback)?;

        todo!()
    }
}
