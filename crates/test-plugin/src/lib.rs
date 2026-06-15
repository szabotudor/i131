use plugin_interface::{
    EngineInterface,
    utils::{SafeResult, SafeString},
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin_entry(interface: &EngineInterface) -> SafeResult<(), SafeString> {
    SafeResult::ok(())
}
