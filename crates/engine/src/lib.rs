pub mod systems;

use std::sync::{Arc, Condvar, Mutex, MutexGuard};

pub use math131;
pub use renderer131;
pub use window131;

use crate::systems::{SystemError, SystemManager};

#[derive(Default, Clone)]
pub struct EngineState {
    pub ready: bool,
    pub running: bool,
    pub playing: bool,
}
pub struct EngineStateHandler {
    mutex: Mutex<EngineState>,
    cond: Condvar,
}

impl Default for EngineStateHandler {
    fn default() -> Self {
        Self {
            mutex: Mutex::new(EngineState::default()),
            cond: Default::default(),
        }
    }
}
impl EngineStateHandler {
    pub fn lock(&self) -> Result<MutexGuard<'_, EngineState>, SystemError> {
        Ok(self.mutex.lock()?)
    }

    pub fn wait_while<F: Fn(&EngineState) -> bool>(
        &self,
        f: F,
    ) -> Result<MutexGuard<'_, EngineState>, SystemError> {
        let lock = self.cond.wait_while(self.lock()?, |state| (f)(state))?;
        Ok(lock)
    }
}

#[derive(Default)]
pub struct I131 {
    systems: SystemManager,
    state: EngineStateHandler,
}

unsafe impl Sync for I131 {}
unsafe impl Send for I131 {}

impl I131 {
    pub fn new() -> Arc<Self> {
        Arc::new_cyclic(|engine| Self {
            systems: SystemManager::new(engine.clone()),
            state: EngineStateHandler::default(),
        })
    }

    pub fn systems(&self) -> &SystemManager {
        &self.systems
    }
}
