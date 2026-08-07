pub mod builtin;
pub mod systems;

use crate::systems::{OptionSystemError, SystemData, SystemError, SystemId, ThreadData};
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, Weak, atomic::AtomicUsize},
};

pub use math131;
pub use renderer131;
pub use window131;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum EngineState {
    #[default]
    Uninitialized,
    Initialized,
    InEditor,
    Running,
    Stopped,
}

#[derive(Debug)]
pub enum TicksPerSecond {
    /// No TPS requirement, run as fast as possible
    FullSpeed,
    /// Prefer set TPS. Log a warning if tick takes too long and thread falls behind
    Prefer(f32),
    /// Require set TPS. Throw an error if tick takes too long and thread falls behind consistently
    /// (more than `threshold`% of the time. Set `threshold` to 0.0 to instantly throw the moment
    /// that the required tick rate isn't met)
    Require { requirement: f32, threshold: f32 },
}

pub trait Thread131
where
    Self: Sized + Send + Sync,
{
    const NAME: &'static str;
    const TPS: TicksPerSecond;
    const AFFINITY: AffinityFor<Self> = AffinityFor::<Self>::new();

    /// Create a new thread
    fn new() -> Self;
}

pub struct AffinityFor<T: Thread131> {
    _marker: PhantomData<T>,
}
#[allow(
    clippy::new_without_default,
    reason = "We need const. There is nothing Default helps with here"
)]
impl<T: Thread131> AffinityFor<T> {
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData::<T> {},
        }
    }
}

/// The engine's main thread, called "Main"
/// Runs as fast as possible (0.0 TPS)
pub struct MainThread {}
impl Thread131 for MainThread {
    const NAME: &'static str = "Main";
    const TPS: TicksPerSecond = TicksPerSecond::FullSpeed;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
    }
}

pub(crate) enum SystemOp {
    Create(SystemData),
    Destroy,
}
pub(crate) struct EngineData {
    system_op_queue: HashMap<SystemId, SystemOp>,
    thread_data: HashMap<&'static str, Arc<RwLock<ThreadData>>>,
    main_thread: Arc<RwLock<ThreadData>>,
    all_systems: HashMap<SystemId, (&'static str, Arc<RwLock<SystemData>>)>,
    lock_order: Vec<SystemId>,
    /// Will be incremented by each thread at the end of their ticks
    /// Engine will reset at end of frame when every thread is done
    state: EngineState,
}
impl EngineData {
    pub(crate) fn get_thread_data(&self, name: &'static str) -> Option<&Arc<RwLock<ThreadData>>> {
        match name {
            "Main" => Some(&self.main_thread),
            other => self.thread_data.get(other),
        }
    }
}

pub struct I131 {
    engine: Weak<Self>,
    state: (Mutex<EngineData>, Condvar),
}

unsafe impl Send for I131 {}
unsafe impl Sync for I131 {}

impl I131 {
    pub fn new() -> Result<Arc<Self>, SystemError> {
        let engine = Arc::new_cyclic(|engine| Self {
            engine: engine.clone(),
            state: (
                Mutex::new(EngineData {
                    system_op_queue: HashMap::new(),
                    thread_data: HashMap::new(),
                    main_thread: Arc::default(),
                    all_systems: HashMap::new(),
                    lock_order: Vec::default(),
                    state: EngineState::default(),
                }),
                Condvar::new(),
            ),
        });

        Ok(engine)
    }

    pub fn main_loop(&self) -> Result<(), SystemError> {
        // If any init code needs to run, it should be here

        let thread_data = {
            let mut state = self.lock()?;
            state.state = EngineState::Running;
            state.main_thread.clone()
        };

        let engine = self
            .engine
            .upgrade()
            .ok_or_system_error(SystemError::InvalidEngine)?;

        loop {
            let state = {
                let lock = self.lock()?;
                if lock.all_systems.is_empty() && lock.state == EngineState::Stopped {
                    println!("Stopping engine");
                    break;
                }
                lock.state
            };
            self.notify_all();

            Self::run_thread_tick::<MainThread>(&engine, state, &thread_data)?;

            {
                self.process_create_and_destroy_queues()?;
            }
            self.notify_all();
        }

        Ok(())
    }

    pub fn request_immediate_shutdown(&self) -> Result<(), SystemError> {
        let systems = self.lock()?.all_systems.keys().cloned().collect::<Vec<_>>();
        self.destroy_systems(systems.into_iter())?;

        Ok(())
    }

    pub(crate) fn wait_while<F: FnMut(&mut EngineData) -> bool>(
        &self,
        f: F,
    ) -> Result<MutexGuard<'_, EngineData>, SystemError> {
        let lock = self.state.1.wait_while(self.state.0.lock()?, f)?;
        Ok(lock)
    }
    pub(crate) fn wait_until<F: FnMut(&mut EngineData) -> bool>(
        &self,
        mut f: F,
    ) -> Result<MutexGuard<'_, EngineData>, SystemError> {
        let lock = self.state.1.wait_while(self.state.0.lock()?, |d| !f(d))?;
        Ok(lock)
    }
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, EngineData>, SystemError> {
        Ok(self.state.0.lock()?)
    }
    pub(crate) fn notify_all(&self) {
        self.state.1.notify_all();
    }
}
