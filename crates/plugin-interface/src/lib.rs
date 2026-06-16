use crate::utils::SafeString;
use std::ffi::c_void;

pub mod utils;

#[repr(C)]
pub struct PluginInfo {
    pub name: SafeString,
}

#[repr(C)]
pub struct EngineInterface {
    engine: *const c_void,
}
