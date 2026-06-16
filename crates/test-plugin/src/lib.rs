use plugin_interface::{
    EngineInterface, PluginInfo,
    utils::{SafeResult, SafeString},
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin_metadata() -> PluginInfo {
    PluginInfo {
        name: "TestPlugin".into(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_entry(interface: &EngineInterface) -> SafeResult<(), SafeString> {
    SafeResult::ok(())
}
