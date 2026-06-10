use std::{
    any::TypeId,
    fmt::Debug,
    sync::{Arc, PoisonError, RwLock},
    thread::{JoinHandle, ThreadId},
};

use crate::{EngineState, I131};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Engine singleton is invalid")]
    InvalidEngine,

    #[error("Engine state is invalid: {0}")]
    InvalidEngineState(String),

    #[error("System doesn't exist: {0:?}")]
    MissingSystem(TypeId),

    #[error("Cyclic dependency detected duing system scheduling. Affected systems: {0:?}")]
    SystemCyclicDependency(Vec<TypeId>),

    #[error("Issue encountered in system thread: {0:?}")]
    SystemThreadError(String),

    #[error("Arc error: {0}")]
    ArcError(String),

    #[error("Lock is poisoned: {0}")]
    LockPoisonError(String),
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

impl<T> From<PoisonError<T>> for SystemError {
    fn from(value: PoisonError<T>) -> Self {
        SystemError::LockPoisonError(value.to_string())
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
    systems: Vec<Arc<dyn System>>,
    join_handle: JoinHandle<Result<(), SystemError>>,
}

impl Debug for ThreadData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadData")
            .field("join_handle", &self.join_handle)
            .finish()
    }
}

pub trait System
where
    Self: Send + Sync,
{
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
    fn thread_tick(thread_id: &ThreadId, engine: &Arc<I131>) -> Result<(), SystemError> {
        let thread_data = {
            let engine_data = engine.lock()?;
            engine_data
                .thread_data
                .get(thread_id)
                .ok_or_system_error(SystemError::SystemThreadError(format!(
                    "System thread doesn't exist for thread id {thread_id:?}"
                )))?
                .clone()
        };

        todo!("Implement single thread tick: {thread_data:?}")
    }

    /// Contains the thread update function too
    pub(crate) fn create_thread(&self) -> Result<(), SystemError> {
        let engine = self
            .engine
            .upgrade()
            .ok_or_system_error(SystemError::InvalidEngine)?;

        let thread_fn = || -> Result<(), SystemError> {
            let thread_id = std::thread::current().id();
            let engine = engine;

            let lock = engine.wait_while(|data| {
                data.state != EngineState::Running || !data.thread_data.contains_key(&thread_id)
            })?;
            let mut state = lock.state;
            drop(lock);

            while state == EngineState::Running {
                Self::thread_tick(&thread_id, &engine)?;
                let lock = engine.lock()?;
                state = lock.state;
            }

            Ok(())
        };

        let join_handle = std::thread::spawn(thread_fn);
        let thread_id = join_handle.thread().id();

        let thread_data = Arc::new(RwLock::new(ThreadData {
            systems: Vec::default(),
            join_handle,
        }));

        {
            let mut engine_data = self.lock()?;
            engine_data.thread_data.insert(thread_id, thread_data);
        }
        self.notify_all();

        Ok(())
    }

    pub fn compute_system_scheduling(&self) -> Result<(), SystemError> {
        todo!()
    }

    pub fn initialize(&self) -> Result<(), SystemError> {
        {
            let mut lock = self.lock()?;
            if lock.state != EngineState::Uninitialized {
                return Err(SystemError::InvalidEngineState(
                    "Engine cannot be initialized more than once".to_string(),
                ));
            }

            // TODO: Internal system init

            lock.state = EngineState::Initialized;
        }
        self.notify_all();
        Ok(())
    }

    pub fn run(&self) -> Result<(), SystemError> {
        {
            let mut lock = self.lock()?;
            if lock.state != EngineState::Initialized {
                return Err(SystemError::InvalidEngineState(
                    "Cannot run engine before initialization, or after stop".to_string(),
                ));
            }

            // TODO: Internal prepare for run

            lock.state = EngineState::Running;
        }
        self.notify_all();
        Ok(())
    }

    pub fn create_system<T: System>(&self, system: T) -> Result<(), SystemError> {
        let _ = system;
        todo!()
    }

    pub fn system<T: System + 'static>(&self) -> Result<&T, SystemError> {
        todo!()
    }

    pub fn destroy(&mut self) -> Result<(), SystemError> {
        todo!()
    }
}
