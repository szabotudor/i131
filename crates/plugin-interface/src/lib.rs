use crate::utils::SafeString;
use std::ffi::c_void;

pub mod utils;

#[repr(C)]
pub struct PluginInfo {
    pub name: SafeString,
}

#[repr(C)]
pub struct EngineInterfaceData {
    pub engine: *const c_void,
}
#[repr(C)]
pub struct EngineInterface {
    data: EngineInterfaceData,
}
impl EngineInterface {
    pub fn new(data: EngineInterfaceData) -> Self {
        Self { data }
    }
}
