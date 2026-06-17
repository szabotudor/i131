use std::ffi::c_void;

use crate::{
    EngineInterface,
    utils::{SafeError, SafeResult, SafeString, SafeVec},
};

pub extern "C" fn system_initialize(
    system: *const c_void,
    engine: &EngineInterface,
) -> SafeResult<(), SafeError> {
    todo!()
}
pub extern "C" fn system_begin_play(
    system: *const c_void,
    engine: &EngineInterface,
) -> SafeResult<(), SafeError> {
    todo!()
}
pub extern "C" fn system_update(
    system: *const c_void,
    engine: &EngineInterface,
    delta: f32,
) -> SafeResult<(), SafeError> {
    todo!()
}
pub extern "C" fn system_in_editor_update(
    system: *const c_void,
    engine: &EngineInterface,
    delta: f32,
) -> SafeResult<(), SafeError> {
    todo!()
}
pub extern "C" fn system_end_play(
    system: *const c_void,
    engine: &EngineInterface,
) -> SafeResult<(), SafeError> {
    todo!()
}
pub extern "C" fn system_destroy(
    system: *const c_void,
    engine: &EngineInterface,
) -> SafeResult<(), SafeError> {
    todo!()
}
pub extern "C" fn system_dependencies(system: *const c_void) -> SafeVec<SafeString> {
    todo!()
}
pub extern "C" fn system_system_id(system: *const c_void) -> SafeString {
    todo!()
}

type SystemInitializeFn =
    extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeError>;
type SystemBeginPlayFn =
    extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeError>;
type SystemUpdateFn =
    extern "C" fn(*const c_void, &EngineInterface, f32) -> SafeResult<(), SafeError>;
type SystemInEditorUpdateFn =
    extern "C" fn(*const c_void, &EngineInterface, f32) -> SafeResult<(), SafeError>;
type SystemEndPlayFn = extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeError>;
type SystemDestroyFn = extern "C" fn(*const c_void, &EngineInterface) -> SafeResult<(), SafeError>;
type SystemDependenciesFn = extern "C" fn(*const c_void) -> SafeVec<SafeString>;
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
    /// Initialize the system.
    ///
    /// Only called when the game or editor are opened,
    /// after all dependencies are already successfully initialized.
    fn initialize(&mut self, engine: &EngineInterface) -> Result<(), SafeError>;

    /// Begin play for this system.
    ///
    /// Called when the game begins. Might be called multiple times in the editor.
    /// Each time the game is ran from the editor, this is called.
    fn begin_play(&mut self, engine: &EngineInterface) -> Result<(), SafeError>;

    /// Called every frame while the game is playing.
    fn update(&mut self, engine: &EngineInterface, delta: f32) -> Result<(), SafeError>;

    /// Called every frame while in the editor.
    fn in_editor_update(&mut self, engine: &EngineInterface, delta: f32) -> Result<(), SafeError>;

    /// End play for this system.
    ///
    /// Caled when the game ends. Might be called multiple times in the editor.
    /// Each time the game is stopped in the editor, this is called.
    fn end_play(&mut self, engine: &EngineInterface) -> Result<(), SafeError>;

    /// Destroy the system.
    ///
    /// Only called when the game or editor are exited,
    fn destroy(&mut self, engine: &EngineInterface) -> Result<(), SafeError>;

    /// Returns list of dependencies for this system.
    /// Dependencies' update function will be scheduled before this system.
    ///
    /// Only dependencies are available to this system through `engine.system<T>()`
    fn dependencies() -> &'static [String]
    where
        Self: Sized;

    /// This system's ID (unique identifier)
    fn system_id() -> String
    where
        Self: Sized;
}
