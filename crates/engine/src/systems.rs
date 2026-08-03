use crate::{AffinityFor, EngineData, EngineState, I131, Thread131};
use renderer131::RendererError;
use std::{
    any::Any,
    fmt::{Debug, Display},
    sync::{Arc, MutexGuard, PoisonError, RwLock, RwLockWriteGuard},
    thread::JoinHandle,
    time::{SystemTime, SystemTimeError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Engine singleton is invalid")]
    InvalidEngine,

    #[error("Engine state is invalid: {0}")]
    InvalidEngineState(String),

    #[error("System already exists: {0}")]
    SystemAlreadyExists(SystemId),

    #[error("System doesn't exist: \"{0}\"")]
    MissingSystem(SystemId),

    #[error("Missing dependency \"{1}\" for system \"{0}\"")]
    MissingDependency(SystemId, SystemId),

    #[error("Failed to downcast System to correct type: \"{0}\" is not {1}")]
    WrongSystemType(SystemId, &'static str),

    #[error("Cyclic dependency detected duing system scheduling. Affected systems: {0:?}")]
    SystemCyclicDependency(Vec<SystemId>),

    #[error("Issue encountered in system thread: {0:?}")]
    SystemThreadError(String),

    #[error("Arc error: {0}")]
    ArcError(String),

    #[error("Lock is poisoned: {0}")]
    LockPoisonError(String),

    #[error("System time error: {0}")]
    StstemTimeError(#[from] SystemTimeError),

    #[error("Renderer error: {0}")]
    RendererError(#[from] RendererError),
}
pub trait OptionSystemError<T> {
    fn ok_or_system_error(self, err: SystemError) -> Result<T, SystemError>;
}
impl<T> OptionSystemError<T> for Option<T> {
    fn ok_or_system_error(self, err: SystemError) -> Result<T, SystemError> {
        if let Some(opt) = self {
            Ok(opt)
        } else {
            Err(err)
        }
    }
}

impl<T> From<PoisonError<T>> for SystemError {
    fn from(value: PoisonError<T>) -> Self {
        SystemError::LockPoisonError(value.to_string())
    }
}

pub(crate) struct SystemData {
    pub(crate) last_update: SystemTime,
    pub(crate) initialized: bool,
    pub(crate) playing: bool,
    pub(crate) queued_for_destroy: bool,
    pub(crate) destroyed: bool,
    pub(crate) system: Box<dyn System>,
    pub(crate) dependencies: &'static [SystemId],
    pub(crate) system_id: SystemId,
    pub(crate) affinity: &'static str,
}
unsafe impl Send for SystemData {}
unsafe impl Sync for SystemData {}
impl SystemData {
    pub(crate) fn new<T: System + 'static>(system: T, affinity: &'static str) -> Self {
        Self {
            last_update: SystemTime::now(),
            initialized: false,
            playing: false,
            queued_for_destroy: false,
            destroyed: false,
            system: Box::new(system),
            dependencies: T::dependencies(),
            system_id: T::system_id(),
            affinity,
        }
    }
}

#[derive(Default)]
pub(crate) struct ThreadData {
    system_data: Vec<Arc<RwLock<SystemData>>>,
    join_handle: Option<JoinHandle<Result<(), SystemError>>>,
}
unsafe impl Sync for ThreadData {}
unsafe impl Send for ThreadData {}

impl Debug for ThreadData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadData")
            .field("join_handle", &self.join_handle)
            .finish()
    }
}

#[derive(PartialEq, Eq, Default, Debug, Clone, Copy, Hash)]
pub struct SystemId(pub &'static str);
impl Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait System
where
    Self: Send + Sync + Any,
{
    /// Initialize the system.
    ///
    /// Only called when the game or editor are opened,
    /// after all dependencies are already successfully initialized.
    fn initialize(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Begin play for this system.
    ///
    /// Called when the game begins. Might be called multiple times in the editor.
    /// Each time the game is ran from the editor, this is called.
    fn begin_play(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Called every frame while the game is playing.
    fn update(&mut self, engine: &I131, delta: f32) -> Result<(), SystemError>;

    /// Called every frame while in the editor.
    fn in_editor_update(&mut self, engine: &I131, delta: f32) -> Result<(), SystemError>;

    /// End play for this system.
    ///
    /// Caled when the game ends. Might be called multiple times in the editor.
    /// Each time the game is stopped in the editor, this is called.
    fn end_play(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Destroy the system.
    ///
    /// Only called when the game or editor are exited,
    fn destroy(&mut self, engine: &I131) -> Result<(), SystemError>;

    /// Returns list of dependencies for this system.
    /// Dependencies' update function will be scheduled before this system.
    ///
    /// Only dependencies are available to this system through `engine.system<T>()`
    fn dependencies() -> &'static [SystemId]
    where
        Self: Sized;

    /// This system's ID (unique identifier)
    fn system_id() -> SystemId
    where
        Self: Sized;
}

impl dyn System {
    pub fn downcast_ref<T: System + 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<T: System + 'static>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
    }
}

impl I131 {
    fn thread_tick(
        engine: &Arc<I131>,
        engine_state: EngineState,
        mut systems: Vec<RwLockWriteGuard<'_, SystemData>>,
    ) -> Result<(), SystemError> {
        while let Some(mut system_data) = systems.pop() {
            let engine: &I131 = engine;

            if (engine_state == EngineState::Initialized
                || engine_state == EngineState::InEditor
                || engine_state == EngineState::Running)
                && !system_data.initialized
            {
                system_data.system.initialize(engine)?;
                system_data.initialized = true;
            }

            if engine_state == EngineState::Running && !system_data.playing {
                system_data.system.begin_play(engine)?;
                system_data.playing = true;
            } else if engine_state != EngineState::Running && system_data.playing {
                system_data.system.end_play(engine)?;
                system_data.playing = false;
            }

            if engine_state == EngineState::Running {
                let now = SystemTime::now();
                let delta_time = now.duration_since(system_data.last_update)?;
                let delta = delta_time.as_secs_f32();
                system_data.last_update = now;
                system_data.system.update(engine, delta)?;
            } else if engine_state == EngineState::InEditor {
                let now = SystemTime::now();
                let delta_time = now.duration_since(system_data.last_update)?;
                let delta = delta_time.as_secs_f32();
                system_data.last_update = now;
                system_data.system.in_editor_update(engine, delta)?;
            }

            if engine_state == EngineState::Stopped || system_data.queued_for_destroy {
                if system_data.playing {
                    system_data.system.end_play(engine)?;
                    system_data.playing = false;
                }
                if system_data.initialized {
                    system_data.system.destroy(engine)?;
                    system_data.initialized = false;
                }
            }
        }

        Ok(())
    }

    /// Contains the thread update function too
    pub fn create_thread<ST: Thread131 + 'static>(
        &self,
        thread: ST,
        state: &mut MutexGuard<'_, EngineData>,
    ) -> Result<(), SystemError> {
        let engine = self
            .engine
            .upgrade()
            .ok_or_system_error(SystemError::InvalidEngine)?;

        let name = ST::NAME;
        let thread_fn = || -> Result<(), SystemError> {
            let thread_id = std::thread::current().id();
            let engine = engine;
            let thread = thread;

            let (thread_data, mut state) = {
                // Wait until engine is running
                let engine_data = engine.wait_while(|data| {
                    data.state != EngineState::Running || !data.thread_data.contains_key(ST::NAME)
                })?;
                let thread_data = engine_data
                    .thread_data
                    .get(ST::NAME)
                    .ok_or_system_error(SystemError::SystemThreadError(format!(
                        "System thread doesn't exist for thread id {thread_id:?}"
                    )))?
                    .clone();

                (thread_data, engine_data.state)
            };

            while state == EngineState::Running {
                state = engine.lock()?.state;
            }

            Ok(())
        };

        let join_handle = std::thread::spawn(thread_fn);

        let thread_data = Arc::new(RwLock::new(ThreadData {
            system_data: Vec::default(),
            join_handle: Some(join_handle),
        }));

        state.thread_data.insert(name, thread_data);

        Ok(())
    }

    pub fn create_system<T: System + 'static, ST: Thread131 + 'static>(
        &self,
        system: T,
        #[expect(unused_variables, reason = "Empty marker used for type info")]
        affinity: AffinityFor<ST>,
    ) -> Result<(), SystemError> {
        let mut state = self.lock()?;
        if state.get_thread_data(ST::NAME).is_none() {
            self.create_thread(ST::new(), &mut state)?;
        }

        let system_data = SystemData::new(system, ST::NAME);

        if state.system_create_queue.contains_key(&T::system_id())
            || state.all_systems.contains_key(&T::system_id())
        {
            return Err(SystemError::SystemAlreadyExists(T::system_id()));
        }

        state
            .system_create_queue
            .insert(T::system_id(), system_data);

        Ok(())
    }
    pub fn destroy_system(&self, system_id: SystemId) -> Result<(), SystemError> {
        let mut state = self.lock()?;

        if !state.all_systems.contains_key(&system_id)
            || state.system_destroy_queue.contains(&system_id)
        {
            return Err(SystemError::MissingSystem(system_id));
        }

        state.system_destroy_queue.insert(system_id);

        Ok(())
    }
    pub fn destroy_systems<I: IntoIterator<Item = SystemId>>(
        &self,
        system_ids: I,
    ) -> Result<(), SystemError> {
        let mut state = self.lock()?;

        let system_ids = system_ids.into_iter().collect::<Vec<_>>();

        let any_missing_system = system_ids
            .iter()
            .find(|id| {
                !state.all_systems.contains_key(id) || state.system_destroy_queue.contains(id)
            })
            .cloned();
        if let Some(system_id) = any_missing_system {
            return Err(SystemError::MissingSystem(system_id));
        }

        state.system_destroy_queue.extend(system_ids);

        Ok(())
    }

    pub(crate) fn process_create_and_destroy_queues(&self) -> Result<(), SystemError> {
        let mut systems_to_destroy = vec![];
        {
            let mut state = self.lock()?;

            let create_queue = state.system_create_queue.drain().collect::<Vec<_>>();
            let destroy_queue = state.system_destroy_queue.drain().collect::<Vec<_>>();

            for (sys_id, system_data) in create_queue {
                let thread_name = system_data.affinity;
                let thread = state.get_thread_data(thread_name).ok_or_system_error(
                    SystemError::SystemThreadError(format!("Thread {thread_name} doesn't exist")),
                )?;

                let slot = {
                    let mut thread_data = thread.write()?;
                    thread_data
                        .system_data
                        .push(Arc::new(RwLock::new(system_data)));

                    thread_data.system_data.len() - 1
                };

                state.all_systems.insert(sys_id, (thread_name, slot));
            }

            for sys_id in destroy_queue {
                let (thread_name, slot) = *state
                    .all_systems
                    .get(&sys_id)
                    .ok_or_system_error(SystemError::MissingSystem(sys_id))?;
                let thread = state.get_thread_data(thread_name).ok_or_system_error(
                    SystemError::SystemThreadError(format!("Thread {thread_name} doesn't exist")),
                )?;
                let systems_to_change = {
                    let mut thread_data = thread.write()?;

                    let system = thread_data.system_data.remove(slot);
                    systems_to_destroy.push(system);

                    thread_data.system_data.split_at(slot).1.to_vec()
                };

                for system in systems_to_change {
                    let system = system.read()?;
                    let system_id = system.system_id;
                    let (_, slot) = state
                        .all_systems
                        .get_mut(&system_id)
                        .ok_or_system_error(SystemError::MissingSystem(system_id))?;
                    *slot -= 1;
                }

                state.all_systems.remove(&sys_id);
            }

            if state.all_systems.is_empty() {
                state.state = EngineState::Stopped;
            }
        }

        for system in systems_to_destroy {
            let mut system_data = system.write()?;

            if !system_data.destroyed {
                if system_data.playing {
                    system_data.system.end_play(self)?;
                    system_data.playing = false;
                }
                if system_data.initialized {
                    system_data.system.destroy(self)?;
                    system_data.initialized = false;
                }

                system_data.destroyed = true;
            }
        }

        Ok(())
    }

    pub fn system<T: System + 'static>(&self, system_id: &SystemId) -> Result<&T, SystemError> {
        todo!()
    }
}
