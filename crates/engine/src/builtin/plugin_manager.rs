use crate::{
    I131,
    systems::{System, SystemError},
};
use libloading::{Library, Symbol};
use plugin_interface::{
    EngineInterface, EngineInterfaceData, PluginInfo,
    systems::SystemVTable,
    utils::{SafeResult, SafeString},
};
use std::{collections::HashMap, os::raw::c_void, path::Path, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Error while loading library: {0}")]
    LibraryLoadError(#[from] libloading::Error),

    #[error("Error while initializing plugin: {0}")]
    PluginInitError(String),

    #[error("Invalid engine")]
    InvalidEngineError,
}
pub trait OptionPluginError<T> {
    fn ok_or_plugin_error(self, err: PluginError) -> Result<T, PluginError>;
}
impl<T> OptionPluginError<T> for Option<T> {
    fn ok_or_plugin_error(self, err: PluginError) -> Result<T, PluginError> {
        if let Some(opt) = self {
            Ok(opt)
        } else {
            Err(err)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_create_system(
    engine: *const c_void,
    system_id: SystemVTable,
) -> SafeResult<(), SafeString> {
    todo!()
}

type PluginMetadataFn = extern "C" fn() -> PluginInfo;
type PluginEntryFn = extern "C" fn(&EngineInterface) -> SafeResult<(), SafeString>;
pub(crate) struct Plugin {
    lib: Library,
    engine_hold: Arc<I131>,
    interface: EngineInterface,
    plugin_metadata: Symbol<'static, PluginMetadataFn>,
    plugin_entry: Symbol<'static, PluginEntryFn>,
}
impl Plugin {
    fn new(path: &Path, engine: &I131) -> Result<Self, PluginError> {
        unsafe {
            let lib = Library::new(path)?;
            let plugin_metadata = std::mem::transmute::<_, Symbol<'static, PluginMetadataFn>>(
                lib.get::<extern "C" fn() -> PluginInfo>("plugin_metadata")?,
            );
            let plugin_entry = std::mem::transmute::<_, Symbol<'static, PluginEntryFn>>(
                lib.get::<extern "C" fn(&EngineInterface) -> SafeResult<(), SafeString>>(
                    "plugin_entry",
                )?,
            );

            let engine_hold = engine
                .engine
                .upgrade()
                .ok_or_plugin_error(PluginError::InvalidEngineError)?;

            let engine = engine as *const I131 as *const c_void;
            let interface = EngineInterface::new(EngineInterfaceData {
                engine,
                engine_create_system,
            });

            Ok(Self {
                lib,
                engine_hold,
                interface,
                plugin_metadata,
                plugin_entry,
            })
        }
    }

    fn plugin_metadata(&self) -> PluginInfo {
        (self.plugin_metadata)()
    }
    fn plugin_entry(&self) -> Result<(), PluginError> {
        (self.plugin_entry)(&self.interface)
            .to_result()
            .map_err(|err| PluginError::PluginInitError(err.to_string()))
    }
}

#[derive(Default)]
pub(crate) struct PluginManager {
    plugins: HashMap<String, Plugin>,
}
unsafe impl Sync for PluginManager {}
unsafe impl Send for PluginManager {}

#[cfg(target_os = "linux")]
const LIB_EXT: &str = "so";
#[cfg(target_os = "windows")]
const LIB_EXT: &str = "dll";

impl System for PluginManager {
    fn initialize(&mut self, engine: &crate::I131) -> Result<(), crate::systems::SystemError> {
        let lib_files = engine
            .plugin_search_path
            .read_dir()?
            .filter(|path| {
                path.as_ref()
                    .is_ok_and(|path| path.path().extension().is_some_and(|ext| ext == LIB_EXT))
                    || path.is_err()
            })
            .map(|path| Ok(path?.path()))
            .collect::<Result<Vec<_>, SystemError>>()?;

        for lib_file in lib_files {
            let plugin = Plugin::new(&lib_file, engine)?;
            let metadata = plugin.plugin_metadata();
            plugin.plugin_entry()?;
            self.plugins.insert(metadata.name.to_string(), plugin);
        }

        Ok(())
    }

    fn begin_play(&mut self, _: &crate::I131) -> Result<(), crate::systems::SystemError> {
        Ok(())
    }

    fn update(&mut self, _: &crate::I131, _: f32) -> Result<(), crate::systems::SystemError> {
        Ok(())
    }

    fn in_editor_update(
        &mut self,
        engine: &crate::I131,
        delta: f32,
    ) -> Result<(), crate::systems::SystemError> {
        // TODO: Check for plugin changes
        let _ = (engine, delta);
        Ok(())
    }

    fn end_play(&mut self, _: &crate::I131) -> Result<(), crate::systems::SystemError> {
        Ok(())
    }

    fn destroy(&mut self, _: &crate::I131) -> Result<(), crate::systems::SystemError> {
        self.plugins.clear();
        Ok(())
    }

    fn dependencies() -> &'static [crate::systems::SystemId]
    where
        Self: Sized,
    {
        &[]
    }

    fn system_id() -> crate::systems::SystemId
    where
        Self: Sized,
    {
        "PluginManager".into()
    }
}
