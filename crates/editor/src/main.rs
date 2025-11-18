use anyhow::{Ok, Result};

use renderer131::Renderer;

fn main() -> Result<()> {
    let renderer = Renderer::new()?;
    dbg!(Renderer::backends());

    Ok(())
}
