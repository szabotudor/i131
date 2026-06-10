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

#[derive(Default)]
pub(crate) struct TickingThreads {
    ticking: HashSet<ThreadId>,
    ticked: HashSet<ThreadId>,
}

pub(crate) struct EngineData {
    thread_data: HashMap<ThreadId, Arc<RwLock<ThreadData>>>,
    all_systems: HashMap<SystemId, Arc<RwLock<SystemData>>>,
    /// Will be incremented by each thread at the end of their ticks
    /// Engine will reset at end of frame when every thread is done
    ticking_threads: Arc<(Mutex<TickingThreads>, Condvar)>,
    state: EngineState,
    scheduler: Box<dyn SystemScheduler>,
    stale: bool,
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
                    thread_data: HashMap::new(),
                    all_systems: HashMap::new(),
                    ticking_threads: Arc::default(),
                    state: EngineState::default(),
                    scheduler,
                    stale: false,
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
        let ticking_threads = self.lock()?.ticking_threads.clone();

        loop {
            ticking_threads.1.notify_all();

            {
                let mut lock = ticking_threads
                    .1
                    .wait_while(ticking_threads.0.lock()?, |t| {
                        let state = self.lock().unwrap();
                        state
                            .thread_data
                            .iter()
                            .all(|(thread_id, _)| t.ticked.contains(thread_id))
                    })?;
                lock.ticked.clear();
            }
        }
        todo!()
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
    pub(crate) fn notify_one(&self) {
        self.state.1.notify_one();
    }
}
