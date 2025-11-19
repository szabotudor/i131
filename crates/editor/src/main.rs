use anyhow::{Context, Ok, Result};

use engine131::{
    renderer131::Renderer,
    window131::{Window, WindowSettings},
};

fn main() -> Result<()> {
    let renderer = Renderer::new(
        Renderer::backends()
            .into_iter()
            .next()
            .context("No rendering backend available")?,
        Window::new(WindowSettings {
            title: "Editor131".to_string(),
        })?,
    )?;
    dbg!(renderer);

    Ok(())
}
