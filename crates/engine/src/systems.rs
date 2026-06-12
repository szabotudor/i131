use std::{
    any::{Any, type_name},
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread::JoinHandle,
    time::{SystemTime, SystemTimeError},
};

use crate::{EngineState, I131};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Engine singleton is invalid")]
    InvalidEngine,

    #[error("Engine state is invalid: {0}")]
    InvalidEngineState(String),

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
}
impl SystemData {
    pub(crate) fn new<T: System + 'static>(system: T) -> Self {
        Self {
            last_update: SystemTime::now(),
            initialized: false,
            playing: false,
            queued_for_destroy: false,
            destroyed: false,
            system: Box::new(system),
            dependencies: T::dependencies(),
        }
    }
}

pub struct SystemAccess {
    data: Arc<RwLock<SystemData>>,
}

pub struct SystemDataReadGuard<'a, T> {
    _guard: RwLockReadGuard<'a, SystemData>,
    value: &'a T,
}
impl<T> Deref for SystemDataReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct SystemDataWriteGuard<'a, T> {
    _guard: RwLockWriteGuard<'a, SystemData>,
    value: &'a mut T,
}
impl<T> Deref for SystemDataWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<T> DerefMut for SystemDataWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl SystemAccess {
    pub(crate) fn new(data: Arc<RwLock<SystemData>>) -> Self {
        Self { data }
    }
    pub fn read<T: System + 'static>(&self) -> Result<SystemDataReadGuard<'_, T>, SystemError> {
        let guard = self.data.read()?;
        let value =
            guard
                .system
                .downcast_ref()
                .ok_or_system_error(SystemError::WrongSystemType(
                    T::system_id(),
                    type_name::<T>(),
                ))?;
        // Borrow erasure is safe because lock owns the data, and lock should be dropped along with
        // SystemDataReadGuard
        //
        // Even if it isn't, the user shouldn't be able to access `value` after they drop
        // SystemDataReadGuard
        let value = value as *const T;

        Ok(SystemDataReadGuard {
            _guard: guard,
            value: unsafe { &*value },
        })
    }

    pub fn write<T: System + 'static>(
        &mut self,
    ) -> Result<SystemDataWriteGuard<'_, T>, SystemError> {
        let mut guard = self.data.write()?;
        let value =
            guard
                .system
                .downcast_mut()
                .ok_or_system_error(SystemError::WrongSystemType(
                    T::system_id(),
                    type_name::<T>(),
                ))?;
        // See comment above
        let value = value as *mut T;

        Ok(SystemDataWriteGuard {
            _guard: guard,
            value: unsafe { &mut *value },
        })
    }
}

pub(crate) struct ThreadData {
    system_data: Vec<Arc<RwLock<SystemData>>>,
    join_handle: JoinHandle<Result<(), SystemError>>,
}

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
        thread_data: &Arc<RwLock<ThreadData>>,
    ) -> Result<(), SystemError> {
        let thread_data = thread_data.read()?;

        let engine_state = engine.lock()?.state;

        for system_data in &thread_data.system_data {
            let mut system_data = system_data.write()?;
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
    pub(crate) fn create_thread(&self) -> Result<(), SystemError> {
        let engine = self
            .engine
            .upgrade()
            .ok_or_system_error(SystemError::InvalidEngine)?;

        let thread_fn = || -> Result<(), SystemError> {
            let thread_id = std::thread::current().id();
            let engine = engine;

            let (thread_data, mut state) = {
                // Wait until engine is running
                let engine_data = engine.wait_while(|data| {
                    data.state != EngineState::Running || !data.thread_data.contains_key(&thread_id)
                })?;
                let thread_data = engine_data
                    .thread_data
                    .get(&thread_id)
                    .ok_or_system_error(SystemError::SystemThreadError(format!(
                        "System thread doesn't exist for thread id {thread_id:?}"
                    )))?
                    .clone();

                (thread_data, engine_data.state.clone())
            };
            engine.notify_all();

            while state == EngineState::Running {
                // Wait until the engine signals the next frame to start
                {
                    let mut lock = engine.wait_while(|data| {
                        data.ticking_threads.ticking.contains(&thread_id)
                            || !data.ticking_threads.allow_new_frame
                    })?;
                    lock.ticking_threads.ticking.insert(thread_id);
                }
                engine.notify_all();

                Self::thread_tick(&engine, &thread_data)?;

                {
                    let mut lock = engine.lock()?;
                    lock.ticking_threads.ticked.insert(thread_id);
                    state = lock.state;
                }
                engine.notify_all();
            }

            Ok(())
        };

        let join_handle = std::thread::spawn(thread_fn);
        let thread_id = join_handle.thread().id();

        let thread_data = Arc::new(RwLock::new(ThreadData {
            system_data: Vec::default(),
            join_handle,
        }));

        {
            let mut engine_data = self.lock()?;
            engine_data.thread_data.insert(thread_id, thread_data);
        }
        self.notify_all();

        Ok(())
    }

    pub fn initialize(&self) -> Result<(), SystemError> {
        {
            let mut lock = self.lock()?;
            if lock.state != EngineState::Uninitialized {
                return Err(SystemError::InvalidEngineState(
                    "Engine cannot be initialized more than once".to_string(),
                ));
            }

            // TODO: Internal system init
            // Will add built-in systems here

            lock.state = EngineState::Initialized;
        }
        self.notify_all();
        Ok(())
    }

    pub(crate) fn run(&self) -> Result<(), SystemError> {
        {
            let mut lock = self.lock()?;
            if lock.state != EngineState::Initialized {
                return Err(SystemError::InvalidEngineState(
                    "Cannot run engine before initialization, or after stop".to_string(),
                ));
            }

            // TODO: Internal prepare for run

            lock.state = EngineState::Running;
        }
        self.notify_all();
        Ok(())
    }

    pub(crate) fn recompute_schedule(&self) -> Result<(), SystemError> {
        let state = self.wait_until_end_of_frame()?;

        let tree = state
            .all_systems
            .iter()
            .map(
                |(sys, deps)| -> Result<(SystemId, HashSet<SystemId>), SystemError> {
                    let deps = deps
                        .read()?
                        .dependencies
                        .iter()
                        .copied()
                        .collect::<HashSet<_>>();
                    Ok((*sys, deps))
                },
            )
            .collect::<Result<HashMap<SystemId, HashSet<SystemId>>, SystemError>>()?;

        let scheduled_threads = state.scheduler.schedule(&tree, state.thread_data.len())?;

        for (schedule, (_, thread_data)) in scheduled_threads.iter().zip(state.thread_data.iter()) {
            let mut thread_data = thread_data.write()?;
            let systems = schedule
                .iter()
                .map(|sys| {
                    state
                        .all_systems
                        .get(sys)
                        .cloned()
                        .ok_or_system_error(SystemError::MissingSystem(*sys))
                })
                .collect::<Result<Vec<_>, SystemError>>()?;
            thread_data.system_data = systems;
        }

        Ok(())
    }

    pub fn create_system<T: System + 'static>(&self, system: T) -> Result<(), SystemError> {
        let mut state = self.lock()?;
        let system_data = SystemData::new(system);

        state
            .system_create_queue
            .push((system_data, T::system_id()));

        Ok(())
    }
    pub fn destroy_system(&self, system_id: SystemId) -> Result<(), SystemError> {
        let mut state = self.lock()?;
        state.system_destroy_queue.push(system_id);

        Ok(())
    }
    pub(crate) fn process_create_and_destroy_queues(&self) -> Result<(), SystemError> {
        {
            let mut state = self.wait_until_end_of_frame()?;

            let create_queue = state.system_create_queue.drain(..).collect::<Vec<_>>();
            for (system_data, sys_id) in create_queue {
                state
                    .all_systems
                    .insert(sys_id, Arc::new(RwLock::new(system_data)));
            }

            let destroy_queue = state.system_destroy_queue.drain(..).collect::<Vec<_>>();
            for sys_id in destroy_queue {
                {
                    let mut system_data = state
                        .all_systems
                        .get(&sys_id)
                        .ok_or_system_error(SystemError::MissingSystem(sys_id))?
                        .write()?;
                    if !system_data.destroyed {
                        if system_data.playing {
                            system_data.system.end_play(self)?;
                        }
                        if system_data.initialized {
                            system_data.system.destroy(self)?;
                        }
                    }
                }
                state.all_systems.remove(&sys_id);
            }

            if state.all_systems.is_empty() {
                state.state = EngineState::Stopped;
            }
        }

        self.recompute_schedule()?;
        Ok(())
    }

    pub fn system<T: System + 'static>(
        &self,
        system_id: &SystemId,
    ) -> Result<SystemAccess, SystemError> {
        let state = self.lock()?;
        let system = state
            .all_systems
            .get(system_id)
            .ok_or_system_error(SystemError::MissingSystem(*system_id))?
            .clone();

        Ok(SystemAccess::new(system))
    }
}
