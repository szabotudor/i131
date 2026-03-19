use std::{
    fmt::Debug,
    sync::{Arc, Mutex, PoisonError, RwLock, Weak},
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

pub trait System: Send + Sync {
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
    systems: Vec<Arc<RwLock<SystemBox>>>,
}

#[derive(Default)]
pub struct SystemManager {
    thread_data: Vec<Arc<Mutex<SystemThreadData>>>,
    threads: Vec<JoinHandle<Result<(), SystemError>>>,
    engine: Weak<I131>,
}

impl SystemManager {
    pub fn new(engine: Weak<I131>) -> Self {
        Self {
            thread_data: Default::default(),
            threads: Default::default(),
            engine,
        }
    }

    fn worker_tick(
        thread_data_mutex: &Arc<Mutex<SystemThreadData>>,
        engine: &Weak<I131>,
    ) -> Result<bool, SystemError> {
        let mut systems = thread_data_mutex.lock()?.systems.clone();
        let engine = engine.upgrade().ok_or(SystemError::InvalidEngine)?;

        for system_arc in &mut systems {
            let mut system = system_arc.write()?;
            if !system.initialized {
                system.system.initialize(&engine)?;
                system.initialized = true;
            }
            // TODO: system update and begin/end play
        }

        Ok(engine.state.lock()?.running)
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
        let engine = self.engine.clone();

        let worker_thread_fn = || -> Result<(), SystemError> {
            let thread_data_mutex = thread_data_mutex;
            let engine = engine;

            let engine_arc = engine.upgrade().ok_or(SystemError::InvalidEngine)?;
            let state = engine_arc.state.wait_while(|state| !state.ready)?;

            let mut running = state.running;

            drop(state);
            drop(engine_arc);

            while running {
                running = Self::worker_tick(&thread_data_mutex, &engine)?;
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
                existing.systems.push(Arc::new(RwLock::new(SystemBox {
                    system: Box::new(sys),
                    initialized: false,
                })));
            } else {
                let new_thread_data = self.create_worker_thread(thread_name)?;
                let mut new_thread_data = new_thread_data.lock()?;
                new_thread_data
                    .systems
                    .push(Arc::new(RwLock::new(SystemBox {
                        system: Box::new(sys),
                        initialized: false,
                    })));
            }
        }
        Ok(())
    }
}
