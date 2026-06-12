pub mod builtin;
pub mod schedulers;
pub mod systems;

use std::{
    collections::{HashMap, HashSet},
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
}

unsafe impl Send for I131 {}
unsafe impl Sync for I131 {}

impl I131 {
    pub fn new(
        num_threads: usize,
        scheduler: Box<dyn SystemScheduler>,
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
            }
            self.notify_all();

            {
                // Just need to wait until all threads are done
                let _lock = self.wait_while(|data| {
                    data.thread_data.iter().any(|(thread_id, _)| {
                        !(data.ticking_threads.ticked.contains(thread_id)
                            && data.ticking_threads.ticking.contains(thread_id))
                    })
                })?;
            };

            self.process_create_and_destroy_queues()?;
        }

        Ok(())
    }

    pub(crate) fn wait_until_end_of_frame(
        &self,
    ) -> Result<MutexGuard<'_, EngineData>, SystemError> {
        self.wait_while(|data| {
            !data.ticking_threads.ticking.is_empty() || !data.ticking_threads.ticked.is_empty()
        })
    }
    pub(crate) fn wait_while<F: FnMut(&mut EngineData) -> bool>(
        &self,
        f: F,
    ) -> Result<MutexGuard<'_, EngineData>, SystemError> {
        let lock = self.state.1.wait_while(self.state.0.lock()?, f)?;
        Ok(lock)
    }
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, EngineData>, SystemError> {
        Ok(self.state.0.lock()?)
    }
    pub(crate) fn notify_all(&self) {
        self.state.1.notify_all();
    }
}
