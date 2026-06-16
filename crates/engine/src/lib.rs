pub mod builtin;
pub mod schedulers;
pub mod systems;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, Weak},
    thread::ThreadId,
};

pub use math131;
pub use renderer131;
pub use window131;

use crate::{
    schedulers::SystemScheduler,
    systems::{SystemData, SystemError, SystemId, ThreadData},
};

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum EngineState {
    #[default]
    Uninitialized,
    Initialized,
    InEditor,
    Running,
    Stopped,
}

#[derive(Default, Debug)]
pub(crate) struct TickingThreads {
    ticking: HashSet<ThreadId>,
    ticked: HashSet<ThreadId>,
    locks_acquired: HashSet<ThreadId>,
}

pub(crate) struct EngineData {
    system_create_queue: Vec<(SystemData, SystemId)>,
    system_destroy_queue: Vec<SystemId>,
    thread_data: HashMap<ThreadId, Arc<RwLock<ThreadData>>>,
    all_systems: HashMap<SystemId, Arc<RwLock<SystemData>>>,
    /// Will be incremented by each thread at the end of their ticks
    /// Engine will reset at end of frame when every thread is done
    ticking_threads: TickingThreads,
    state: EngineState,
    scheduler: Box<dyn SystemScheduler>,
}

pub struct I131 {
    engine: Weak<Self>,
    state: (Mutex<EngineData>, Condvar),
    pub(crate) plugin_search_path: PathBuf,
}

unsafe impl Send for I131 {}
unsafe impl Sync for I131 {}

impl I131 {
    pub fn new(
        num_threads: usize,
        scheduler: Box<dyn SystemScheduler>,
        plugin_search_path: PathBuf,
    ) -> Result<Arc<Self>, SystemError> {
        let engine = Arc::new_cyclic(|engine| Self {
            engine: engine.clone(),
            state: (
                Mutex::new(EngineData {
                    system_create_queue: Vec::new(),
                    system_destroy_queue: Vec::new(),
                    thread_data: HashMap::new(),
                    all_systems: HashMap::new(),
                    ticking_threads: TickingThreads::default(),
                    state: EngineState::default(),
                    scheduler,
                }),
                Condvar::new(),
            ),
            plugin_search_path,
        });

        for _ in 0..num_threads {
            engine.create_thread()?;
        }

        Ok(engine)
    }

    pub fn main_loop(&self) -> Result<(), SystemError> {
        self.run()?;
        loop {
            {
                let mut lock = self.lock()?;
                if lock.all_systems.is_empty() && lock.state == EngineState::Stopped {
                    println!("Stopping engine");
                    break;
                }
                lock.ticking_threads.ticked.clear();
                lock.ticking_threads.ticking.clear();
                lock.ticking_threads.locks_acquired.clear();
            }
            self.notify_all();

            {
                // Just need to wait until all threads are done
                let _lock = self.wait_until(|data| {
                    data.thread_data
                        .iter()
                        .all(|(thread_id, _)| data.ticking_threads.ticked.contains(thread_id))
                })?;
            };

            self.process_create_and_destroy_queues()?;
        }

        Ok(())
    }

    pub(crate) fn wait_until_end_of_frame(
        &self,
    ) -> Result<MutexGuard<'_, EngineData>, SystemError> {
        self.wait_until(|data| {
            data.thread_data
                .iter()
                .all(|(thread_id, _)| data.ticking_threads.ticked.contains(thread_id))
        })
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
