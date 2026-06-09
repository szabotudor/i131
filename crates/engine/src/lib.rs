pub mod systems;

use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, Weak},
    thread::ThreadId,
};

pub use math131;
pub use renderer131;
pub use window131;

use crate::systems::{SystemError, ThreadData};

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum EngineState {
    #[default]
    Uninitialized,
    Initialized,
    Running,
    Stopped,
}

#[derive(Default)]
pub(crate) struct EngineData {
    thread_data: HashMap<ThreadId, Arc<RwLock<ThreadData>>>,
    state: EngineState,
}

#[derive(Default)]
pub struct I131 {
    engine: Weak<Self>,
    state: (Mutex<EngineData>, Condvar),
}

unsafe impl Send for I131 {}
unsafe impl Sync for I131 {}

impl I131 {
    pub fn new(num_threads: usize) -> Result<Arc<Self>, SystemError> {
        let engine = Arc::new_cyclic(|engine| Self {
            engine: engine.clone(),
            state: Default::default(),
        });

        for _ in 0..num_threads {
            engine.create_thread()?;
        }

        Ok(engine)
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
