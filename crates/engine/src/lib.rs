pub mod systems;

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

pub use math131;
pub use renderer131;
pub use window131;

use crate::systems::{SystemIndex, ThreadData};

#[derive(Default, Clone, Copy)]
pub enum EngineState {
    #[default]
    Uninitialized,
    Initialized,
    Running,
    BetweenFrame,
}

#[derive(Default)]
pub struct I131 {
    engine: Weak<Self>,
    systems: Vec<Arc<Mutex<ThreadData>>>,
    system_idx: HashMap<TypeId, SystemIndex>,
    state: EngineState,
}

unsafe impl Send for I131 {}
unsafe impl Sync for I131 {}

impl I131 {
    pub fn new() -> Arc<Self> {
        Arc::new_cyclic(|engine| Self {
            engine: engine.clone(),
            systems: Vec::new(),
            system_idx: HashMap::default(),
            state: EngineState::default(),
        })
    }
}
