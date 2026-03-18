pub mod systems;

use std::sync::Arc;

pub use math131;
pub use renderer131;
pub use window131;

use crate::systems::{SystemError, SystemManager};

#[derive(Default)]
pub struct I131 {
    systems: SystemManager,
}

unsafe impl Sync for I131 {}
unsafe impl Send for I131 {}

impl I131 {
    pub fn new() -> Arc<Self> {
        Arc::new_cyclic(|engine| Self {
            systems: SystemManager::new(engine.clone()),
        })
    }

    pub fn init(&self) -> Result<(), SystemError> {
        if self.systems().is_ready()? {
            return Err(SystemError::DoubleInitialization(
                "Already called 'init' on the engine".to_string(),
            ));
        }
        self.systems().mark_ready()
    }

    pub fn systems(&self) -> &SystemManager {
        &self.systems
    }
}
