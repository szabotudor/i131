use std::{any::TypeId, fmt::Debug};

use crate::I131;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("{0}")]
    ArcError(String),

    #[error("System doesn't exist: {0}")]
    MissingSystem(String),

    #[error("{0}")]
    MutexError(String),
}
pub trait OptionSystemError<T> {
    fn ok_or_system_error(self, err: SystemError) -> Result<T, SystemError>;
}
impl<T> OptionSystemError<T> for Option<T> {
    fn ok_or_system_error(self, err: SystemError) -> Result<T, SystemError> {
        if let Some(opt) = self {
            Ok(opt)
        } else {
            Err(err)
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct SystemIndex(i32, i32);

impl SystemIndex {
    pub fn new(thread: i32, system: i32) -> Self {
        Self(thread, system)
    }
}

pub(crate) struct ThreadData {
    systems: Vec<Box<dyn System>>,
}

pub trait System {
    /// Initialize the system.
    ///
    /// Only called when the game or editor are opened,
    /// after all dependencies are already successfully initialized.
    fn initialize(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Begin play for this system.
    ///
    /// Called when the game begins. Might be called multiple times in the editor.
    /// Each time the game is ran from the editor, this is called.
    fn begin_play(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Called every frame while the game is playing.
    fn update(&mut self, engine: &I131, delta: f32) -> Result<(), SystemError>;

    /// Called every frame while in the editor.
    fn in_editor_update(&mut self, engine: &I131, delta: f32) -> Result<(), SystemError>;

    /// End play for this system.
    ///
    /// Caled when the game ends. Might be called multiple times in the editor.
    /// Each time the game is stopped in the editor, this is called.
    fn end_play(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Destroy the system.
    ///
    /// Only called when the game or editor are exited,
    fn destroy(&mut self, engine: &I131) -> Result<(), SystemError>;
}

impl I131 {
    pub fn initialize(&mut self) -> Result<(), SystemError> {
        todo!()
    }

    pub fn create_system<T: System>(&mut self, system: T) -> Result<(), SystemError> {
        let _ = system;
        todo!()
    }

    pub fn system<T: System + 'static>(&self) -> Result<&T, SystemError> {
        let typeid = TypeId::of::<T>();
        let idx = self
            .system_idx
            .get(&typeid)
            .ok_or_system_error(SystemError::MissingSystem(format!("{:?}", typeid)))?;

        let thread =
            self.systems
                .get(idx.0 as usize)
                .ok_or_system_error(SystemError::MissingSystem(format!(
                    "Thread index out of range in system ID: {:?}",
                    typeid
                )))?;

        let thread = thread
            .lock()
            .map_err(|e| SystemError::MutexError(format!("{e:?}")))?;

        let system =
            thread
                .systems
                .get(idx.1 as usize)
                .ok_or_system_error(SystemError::MissingSystem(format!(
                    "System index out of range in system ID: {:?}",
                    typeid
                )))?;

        let system = system.as_ref();

        todo!()
    }

    pub fn destroy(&mut self) -> Result<(), SystemError> {
        todo!()
    }
}
