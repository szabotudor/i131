use plugin_interface::{
    EngineInterface, PluginInfo,
    systems::SystemInterface,
    utils::{SafeError, SafeResult, SafeString},
};

pub struct TestSystem {}

impl SystemInterface for TestSystem {
    fn initialize(&mut self, _: &EngineInterface) -> SafeResult<(), SafeError> {
        println!("Initialize plugin system");
        SafeResult::ok(())
    }

    fn begin_play(&mut self, _: &EngineInterface) -> SafeResult<(), SafeError> {
        println!("Begin play for plugin system");
        SafeResult::ok(())
    }

    fn update(&mut self, _: &EngineInterface, delta: f32) -> SafeResult<(), SafeError> {
        println!("Update plugin system: {delta}");
        SafeResult::ok(())
    }

    fn in_editor_update(&mut self, _: &EngineInterface, delta: f32) -> SafeResult<(), SafeError> {
        println!("Update plugin system: {delta}");
        SafeResult::ok(())
    }

    fn end_play(&mut self, _: &EngineInterface) -> SafeResult<(), SafeError> {
        println!("Eng play for plugin system");
        SafeResult::ok(())
    }

    fn destroy(&mut self, engine: &EngineInterface) -> SafeResult<(), SafeError> {
        println!("Destroy plugin system");
        SafeResult::ok(())
    }

    fn dependencies() -> &'static [SafeString]
    where
        Self: Sized,
    {
        &[]
    }

    fn system_id() -> SafeString
    where
        Self: Sized,
    {
        "TestPlugin".into()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_metadata() -> PluginInfo {
    PluginInfo {
        name: "TestPlugin".into(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_entry(interface: &EngineInterface) -> SafeResult<(), SafeString> {
    let res = main(interface).map_err(|err| SafeString::from(err));
    SafeResult::from(res)
}

fn main(interface: &EngineInterface) -> Result<(), String> {
    interface.create_system(TestSystem {});
    Ok(())
}
