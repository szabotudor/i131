use crate::main;
use plugin_interface::{
    EngineInterface, PluginInfo,
    utils::{SafeError, SafeResult},
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin_entry(interface: &EngineInterface) -> SafeResult<PluginInfo, SafeError> {
    let res = main(interface);
    SafeResult::from(res)
}
