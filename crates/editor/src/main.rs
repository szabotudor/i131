use engine131::{I131, MainThread, Thread131, renderer131::shaders_file};

use crate::editor::Editor;

mod editor;
// Include shaders from shaders.rs
shaders_file!("Vulkan", shaders_vulkan);

trait ResultPrint<T> {
    fn print(self) -> Result<T, i32>;
}
impl<T, E: std::fmt::Display> ResultPrint<T> for Result<T, E> {
    fn print(self) -> Result<T, i32> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                eprintln!("{err}");
                Err(-1)
            }
        }
    }
}

fn main() -> Result<(), i32> {
    // TODO: File isn't used now, but if it's used in the future this line can be removed
    let _ = shaders_vulkan::SHADERS;

    let engine = I131::new().print()?;
    println!("Created engine");

    engine
        .create_system(Editor::new().print()?, MainThread::AFFINITY)
        .print()?;
    println!("Opened editor");

    engine.main_loop().print()?;

    Ok(())
}
