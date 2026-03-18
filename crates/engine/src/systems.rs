use std::{
    fmt::Debug,
    sync::{Arc, Condvar, Mutex, PoisonError, Weak},
    thread::JoinHandle,
};

use crate::I131;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("System depends on another system that doesn't exist: {}", self)]
    MissingDependency(String),
    #[error("System has some invalid configuration: {}", self)]
    InvalidSystem(String),
    #[error("System initialization returned an error: {}", self)]
    InitializationFailed(String),
    #[error("System depends on a missing OS dependency: {}", self)]
    OSError(String),
    #[error("Error while performing atomic operation: {}", self)]
    ThreadingError(String),
    #[error("The engine that created this system doesn't exist anymore (went out of scope)")]
    InvalidEngine,
    #[error(
        "Some component of the engine or a system or systems was attempted to be initialized when it is already in a ready state"
    )]
    DoubleInitialization(String),
}

impl<T> From<PoisonError<T>> for SystemError {
    fn from(value: PoisonError<T>) -> Self {
        Self::ThreadingError(value.to_string())
    }
}

pub trait System: Send {
    /// Static name of the system
    fn name(&self) -> &'static str;

    /// Called as soon as every system is added to the manager and the engine starts
    ///
    /// This function prepares the system to run, but doesn't run it
    fn initialize(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Called before the game starts
    ///
    /// Expect this function to be called multiple times between initialization and destruction of
    /// the system
    fn begin_play(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Called every frame during play
    fn update(&mut self, delta: f32, engine: &I131) -> Result<(), SystemError>;

    /// Called every frame outside of play while the editor is enabled and running
    fn editor_update(&mut self, delta: f32, engine: &I131) -> Result<(), SystemError>;

    /// Called as the game ends
    ///
    /// Expect this function to be called multiple times between initialization and destruction of
    /// the system
    fn end_play(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Called before the engine is closed
    ///
    /// This function releases any dependencies (in-engine and OS)
    fn destroy(&mut self, engine: &I131) -> Result<(), SystemError>;
}

struct SystemBox {
    system: Box<dyn System>,
    initialized: bool,
}

struct SystemThreadData {
    name: String,
    systems: Vec<SystemBox>,
}

#[derive(Default)]
struct SystemManagerCommonData {
    ready: bool,
    running: bool,
    engine: Weak<I131>,
}

#[derive(Default, Debug)]
struct CondvarDataTuple<T> {
    data: Mutex<T>,
    cond: Condvar,
}

#[derive(Default)]
pub struct SystemManager {
    common_data: Arc<CondvarDataTuple<SystemManagerCommonData>>,
    thread_data: Vec<Arc<Mutex<SystemThreadData>>>,
    threads: Vec<JoinHandle<Result<(), SystemError>>>,
}

impl SystemManager {
    pub fn new(engine: Weak<I131>) -> Self {
        Self {
            common_data: Arc::new(CondvarDataTuple {
                data: Mutex::new(SystemManagerCommonData {
                    ready: false,
                    running: false,
                    engine,
                }),
                cond: Default::default(),
            }),
            thread_data: Default::default(),
            threads: Default::default(),
        }
    }

    pub(crate) fn is_ready(&self) -> Result<bool, SystemError> {
        Ok(self.common_data.data.lock()?.ready)
    }
    pub(crate) fn mark_ready(&self) -> Result<(), SystemError> {
        let mut common_data = self.common_data.data.lock()?;
        common_data.ready = true;

        Ok(())
    }

    fn worker_tick(
        thread_data_mutex: &Arc<Mutex<SystemThreadData>>,
        engine: &Weak<I131>,
    ) -> Result<(), SystemError> {
        let mut thread_data = thread_data_mutex.lock()?;
        let Some(engine) = engine.upgrade() else {
            return Err(SystemError::InvalidEngine);
        };

        for system in &mut thread_data.systems {
            if !system.initialized {
                system.system.initialize(&engine)?;
                system.initialized = true;
            }
        }

        Ok(())
    }

    /// Create a worker thread to handle system update functions
    ///
    /// `name`: The name of the system
    fn create_worker_thread(
        &mut self,
        name: String,
    ) -> Result<Arc<Mutex<SystemThreadData>>, SystemError> {
        self.thread_data.push(Arc::new(Mutex::new(SystemThreadData {
            name,
            systems: Vec::default(),
        })));

        let thread_data_mutex = self.thread_data.last().unwrap().clone();
        let last_thread_data = self.thread_data.last().unwrap().clone();
        let common_data_condvar = self.common_data.clone();

        let worker_thread_fn = || -> Result<(), SystemError> {
            let thread_data_mutex = thread_data_mutex;
            let common_data_condvar = common_data_condvar;

            let common_data = common_data_condvar
                .cond
                .wait_while(common_data_condvar.data.lock()?, |d| !d.ready)?;

            let mut running = common_data.running;
            let engine = common_data.engine.clone();

            drop(common_data);

            while running {
                Self::worker_tick(&thread_data_mutex, &engine)?;
                running = common_data_condvar.data.lock()?.running;
            }

            Ok(())
        };

        self.threads.push(std::thread::spawn(worker_thread_fn));

        Ok(last_thread_data)
    }

    pub fn create_system<Sys: System + 'static>(
        &mut self,
        sys: Sys,
        pin_to_thread: Option<String>,
    ) -> Result<(), SystemError> {
        if let Some(thread_name) = pin_to_thread {
            if let Some(mut existing) = self
                .thread_data
                .iter()
                .find_map(|t| {
                    let t = t.lock();
                    match t {
                        Ok(t) => {
                            if t.name == thread_name {
                                Some(Ok(t))
                            } else {
                                None
                            }
                        }
                        Err(e) => Some(Err(e)),
                    }
                })
                .transpose()?
            {
                existing.systems.push(SystemBox {
                    system: Box::new(sys),
                    initialized: false,
                });
            } else {
                let new_thread_data = self.create_worker_thread(thread_name)?;
                let mut new_thread_data = new_thread_data.lock()?;
                new_thread_data.systems.push(SystemBox {
                    system: Box::new(sys),
                    initialized: false,
                });
            }
        }
        Ok(())
    }
}
