use crate::{
    systems::{SystemInterface, SystemVTable},
    utils::{SafeResult, SafeString},
};
use std::ffi::c_void;

pub mod systems;
pub mod utils;

#[repr(C)]
pub struct PluginInfo {
    pub name: SafeString,
}

type EngineCreateSystemFn =
    extern "C" fn(*const c_void, SystemVTable) -> SafeResult<(), SafeString>;
#[repr(C)]
pub struct EngineInterfaceData {
    pub engine: *const c_void,
    pub engine_create_system: EngineCreateSystemFn,
}
#[repr(C)]
pub struct EngineInterface {
    data: EngineInterfaceData,
}
impl EngineInterface {
    pub fn new(data: EngineInterfaceData) -> Self {
        Self { data }
    }

    pub fn create_system<T: SystemInterface>(&self, system: T) -> SafeResult<(), SafeString> {
        todo!()
    }
}
