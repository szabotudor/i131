pub mod meta;

use plugin_interface::{EngineInterface, PluginInfo, systems::SystemInterface, utils::SafeError};

pub enum TestSystemError {
    UnknownError,
}
impl From<TestSystemError> for SafeError {
    fn from(value: TestSystemError) -> Self {
        match value {
            TestSystemError::UnknownError => Self::new(-1, "Unknown error".into()),
        }
    }
}

pub struct TestSystem {}

impl SystemInterface for TestSystem {
    fn initialize(&mut self, _: &EngineInterface) -> Result<(), SafeError> {
        println!("Initialize plugin system");
        Ok(())
    }

    fn begin_play(&mut self, _: &EngineInterface) -> Result<(), SafeError> {
        println!("Begin play for plugin system");
        Ok(())
    }

    fn update(&mut self, _: &EngineInterface, delta: f32) -> Result<(), SafeError> {
        println!("Update plugin system: {delta}");
        Ok(())
    }

    fn in_editor_update(&mut self, _: &EngineInterface, delta: f32) -> Result<(), SafeError> {
        println!("In editor update plugin system: {delta}");
        Ok(())
    }

    fn end_play(&mut self, _: &EngineInterface) -> Result<(), SafeError> {
        println!("End play for plugin system");
        Ok(())
    }

    fn destroy(&mut self, _: &EngineInterface) -> Result<(), SafeError> {
        println!("Destroy plugin system");
        Ok(())
    }

    fn dependencies() -> &'static [String]
    where
        Self: Sized,
    {
        &[]
    }

    fn system_id() -> String
    where
        Self: Sized,
    {
        "TestSystem".to_string()
    }
}

fn main(interface: &EngineInterface) -> Result<PluginInfo, SafeError> {
    interface.create_system(TestSystem {})?;
    Ok(PluginInfo {
        name: "TestPlugin".into(),
    })
}
