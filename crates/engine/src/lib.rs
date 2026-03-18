pub mod systems;

use std::sync::{Arc, atomic::AtomicBool};

pub use math131;
pub use renderer131;
pub use window131;

use crate::systems::SystemManager;

#[derive(Default)]
pub struct I131 {
    systems: SystemManager,
    running: AtomicBool,
}

unsafe impl Sync for I131 {}
unsafe impl Send for I131 {}

impl I131 {
    pub fn new() -> Arc<Self> {
        Arc::new_cyclic(|engine| Self {
            systems: SystemManager::new(engine.clone()),
            running: AtomicBool::new(false),
        })
    }

    pub fn systems(&self) -> &SystemManager {
        &self.systems
    }
}
