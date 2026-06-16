use std::ffi::c_void;

use crate::{
    EngineInterface,
    utils::{SafeResult, SafeString},
};

type SystemInitializeFn =
    extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeString>;
type SystemBeginPlayFn =
    extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeString>;
type SystemUpdateFn =
    extern "C" fn(*const c_void, &EngineInterface, f32) -> SafeResult<(), SafeString>;
type SystemInEditorUpdateFn =
    extern "C" fn(*const c_void, &EngineInterface, f32) -> SafeResult<(), SafeString>;
type SystemEndPlayFn = extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeString>;
type SystemDestroyFn = extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeString>;
type SystemDependenciesFn = extern "C" fn(*const c_void) -> SafeResult<(), SafeString>;
type SystemSystemIdFn = extern "C" fn(*const c_void) -> SafeString;
#[repr(C)]
pub struct SystemVTable {
    system: *const c_void,
    initialize: SystemInitializeFn,
    begin_play: SystemBeginPlayFn,
    update: SystemUpdateFn,
    in_editor_update: SystemInEditorUpdateFn,
    end_play: SystemEndPlayFn,
    destroy: SystemDestroyFn,
    dependencies: SystemDependenciesFn,
    system_id: SystemSystemIdFn,
}

pub trait SystemInterface
where
    Self: Send + Sync,
{
}
