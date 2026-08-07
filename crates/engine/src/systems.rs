use crate::{AffinityFor, EngineData, EngineState, I131, SystemOp, Thread131, TicksPerSecond};
use renderer131::RendererError;
use std::{
    any::{Any, type_name},
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    sync::{Arc, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard},
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

    #[error(
        "System \"{0}\" tried requesting a second context. Only one context is allowed per system"
    )]
    DoubleSystemContext(SystemId),

    #[error("Tried writing to system \"{0}\" that was borrowed as read-only")]
    MutError(SystemId),

    #[error("Arc error: {0}")]
    ArcError(String),

    #[error("Lock is poisoned: {0}")]
    LockPoisonError(String),

    #[error("System time error: {0}")]
    StstemTimeError(#[from] SystemTimeError),

    #[error(
        "Thread {thread} failed to meet required TPS of {requirement}. Behind on {overtime:.0}% of ticks."
    )]
    TpsRequirementError {
        thread: &'static str,
        requirement: f32,
        overtime: f32,
    },

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
    pub(crate) initialized: bool,
    pub(crate) playing: bool,
    pub(crate) queued_for_destroy: bool,
    pub(crate) destroyed: bool,
    pub(crate) system: Box<dyn SystemInterface>,
    pub(crate) system_id: SystemId,
    pub(crate) after: &'static [SystemId],
    pub(crate) before: &'static [SystemId],
    pub(crate) dependencies: &'static [SystemId],
    pub(crate) affinity: &'static str,
}
unsafe impl Send for SystemData {}
unsafe impl Sync for SystemData {}
impl SystemData {
    pub(crate) fn new<T: System + 'static>(system: T, affinity: &'static str) -> Self {
        Self {
            initialized: false,
            playing: false,
            queued_for_destroy: false,
            destroyed: false,
            system: Box::new(system),
            system_id: T::SYSTEM_ID,
            after: T::AFTER,
            before: T::BEFORE,
            dependencies: T::DEPENDENCIES,
            affinity,
        }
    }

    pub(crate) fn request_context(&self, engine: Arc<I131>) -> Result<SystemContext, SystemError> {
        if self.initialized {
            return Err(SystemError::DoubleSystemContext(self.system_id));
        }

        Ok(SystemContext {
            engine: Some(engine),
        })
    }
}

pub(crate) struct ThreadData {
    system_data: HashMap<SystemId, Arc<RwLock<SystemData>>>,
    order: Vec<SystemId>,
    last: SystemTime,
    delta_acc: f32,
    /// Only `None` when used by the main thread
    join_handle: Option<JoinHandle<Result<(), SystemError>>>,
    /// Smoothed fraction of ticks that run behind. Bounded 0..1, never overflows.
    current_overtime_percent: f32,
}
unsafe impl Sync for ThreadData {}
unsafe impl Send for ThreadData {}

impl Default for ThreadData {
    fn default() -> Self {
        Self {
            system_data: HashMap::default(),
            order: Vec::default(),
            last: SystemTime::now(),
            delta_acc: 0.0,
            join_handle: None,
            current_overtime_percent: 0.0,
        }
    }
}

impl Debug for ThreadData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadData")
            .field("join_handle", &self.join_handle)
            .finish()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Debug, Clone, Copy, Hash)]
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
    /// This system's ID (unique identifier)
    const SYSTEM_ID: SystemId;
    /// List of systems that this system should be allowed to access
    /// Cyclic dependencies are not allowed
    const DEPENDENCIES: &'static [SystemId];
    /// Returns list of systems on the same thread to update only after this system.
    const BEFORE: &'static [SystemId];
    /// Returns list of systems on the same thread to update before this system.
    const AFTER: &'static [SystemId];

    /// Initialize the system.
    ///
    /// Only called when the game or editor are opened,
    /// after all dependencies are already successfully initialized.
    fn initialize(&mut self, context: SystemContext) -> Result<(), SystemError>;

    /// Begin play for this system.
    ///
    /// Called when the game begins. Might be called multiple times in the editor.
    /// Each time the game is ran from the editor, this is called.
    fn begin_play(&mut self) -> Result<(), SystemError>;

    /// Called every frame while the game is playing.
    fn update(&mut self, delta: f32) -> Result<(), SystemError>;

    /// Called every frame while in the editor.
    fn in_editor_update(&mut self, delta: f32) -> Result<(), SystemError>;

    /// End play for this system.
    ///
    /// Caled when the game ends. Might be called multiple times in the editor.
    /// Each time the game is stopped in the editor, this is called.
    fn end_play(&mut self) -> Result<(), SystemError>;

    /// Destroy the system.
    ///
    /// Only called when the game or editor are exited,
    fn destroy(&mut self) -> Result<(), SystemError>;
}

pub(crate) trait SystemInterface
where
    Self: Any,
{
    fn initialize(&mut self, context: SystemContext) -> Result<(), SystemError>;
    fn begin_play(&mut self) -> Result<(), SystemError>;
    fn update(&mut self, delta: f32) -> Result<(), SystemError>;
    fn in_editor_update(&mut self, delta: f32) -> Result<(), SystemError>;
    fn end_play(&mut self) -> Result<(), SystemError>;
    fn destroy(&mut self) -> Result<(), SystemError>;
}

impl<T: System> SystemInterface for T {
    fn initialize(&mut self, context: SystemContext) -> Result<(), SystemError> {
        self.initialize(context)
    }

    fn begin_play(&mut self) -> Result<(), SystemError> {
        self.begin_play()
    }

    fn update(&mut self, delta: f32) -> Result<(), SystemError> {
        self.update(delta)
    }

    fn in_editor_update(&mut self, delta: f32) -> Result<(), SystemError> {
        self.in_editor_update(delta)
    }

    fn end_play(&mut self) -> Result<(), SystemError> {
        self.end_play()
    }

    fn destroy(&mut self) -> Result<(), SystemError> {
        self.destroy()
    }
}

impl dyn SystemInterface {
    pub fn downcast_ref<T: SystemInterface + 'static>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
    pub fn downcast_mut<T: SystemInterface + 'static>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut()
    }
}

pub struct SystemContext {
    engine: Option<Arc<I131>>,
}
impl SystemContext {
    pub fn empty() -> Self {
        Self { engine: None }
    }
    pub fn engine(&self) -> Result<&I131, SystemError> {
        Ok(self
            .engine
            .as_ref()
            .ok_or_system_error(SystemError::InvalidEngine)?)
    }
    pub fn lock_request(&mut self) -> LockRequest<'_> {
        LockRequest {
            context: self,
            systems: HashMap::default(),
        }
    }
}

enum RequestAccess {
    Read,
    Write,
}
pub struct LockRequest<'a> {
    context: &'a mut SystemContext,
    systems: HashMap<SystemId, RequestAccess>,
}
impl<'a> LockRequest<'a> {
    pub fn read<T: System>(mut self) -> Result<Self, SystemError> {
        if self
            .systems
            .insert(T::SYSTEM_ID, RequestAccess::Read)
            .is_some()
        {
            return Err(SystemError::SystemAlreadyExists(T::SYSTEM_ID));
        }

        Ok(self)
    }
    pub fn write<T: System>(mut self) -> Result<Self, SystemError> {
        if self
            .systems
            .insert(T::SYSTEM_ID, RequestAccess::Write)
            .is_some()
        {
            return Err(SystemError::SystemAlreadyExists(T::SYSTEM_ID));
        }

        Ok(self)
    }

    pub fn acquire(self) -> Result<LockedSet<'a>, SystemError> {
        let systems = {
            let state = self.context.engine()?.lock()?;
            state
                .lock_order
                .iter()
                .filter_map(|id| {
                    if let Some(access) = self.systems.get(id) {
                        state
                            .all_systems
                            .get(id)
                            .map(|(_, data)| (*id, data.clone(), access))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        let mut entries = HashMap::default();
        for (id, data, access) in systems {
            // Borrow erasure should be safe because we keep a copy of the Arc, so it shouldn't
            // destroy the RwLock before we destroy the guard
            match access {
                RequestAccess::Read => {
                    let erased_entry = unsafe {
                        std::mem::transmute::<_, RwLockReadGuard<'a, SystemData>>(data.read()?)
                    };
                    entries.insert(id, LockedSystemEntry::Read(data, erased_entry));
                }
                RequestAccess::Write => {
                    let erased_entry = unsafe {
                        std::mem::transmute::<_, RwLockWriteGuard<'a, SystemData>>(data.write()?)
                    };
                    entries.insert(id, LockedSystemEntry::Write(data, erased_entry));
                }
            }
        }

        Ok(LockedSet {
            _context: self.context,
            entries,
        })
    }
}

enum LockedSystemEntry<'a> {
    #[expect(
        dead_code,
        reason = "Only kept for safety, system shouldn't be dropped while the guard still holds a reference to it"
    )]
    Read(Arc<RwLock<SystemData>>, RwLockReadGuard<'a, SystemData>),
    #[expect(
        dead_code,
        reason = "Only kept for safety, system shouldn't be dropped while the guard still holds a reference to it"
    )]
    Write(Arc<RwLock<SystemData>>, RwLockWriteGuard<'a, SystemData>),
}
pub struct LockedSet<'a> {
    _context: &'a mut SystemContext,
    entries: HashMap<SystemId, LockedSystemEntry<'a>>,
}
impl<'a> LockedSet<'a> {
    pub fn get<T: System>(&self) -> Result<&T, SystemError> {
        let system = self
            .entries
            .get(&T::SYSTEM_ID)
            .ok_or_system_error(SystemError::MissingSystem(T::SYSTEM_ID))?;

        match system {
            LockedSystemEntry::Read(_, data) => {
                data.system
                    .downcast_ref()
                    .ok_or_system_error(SystemError::WrongSystemType(
                        data.system_id,
                        type_name::<T>(),
                    ))
            }
            LockedSystemEntry::Write(_, data) => {
                data.system
                    .downcast_ref()
                    .ok_or_system_error(SystemError::WrongSystemType(
                        data.system_id,
                        type_name::<T>(),
                    ))
            }
        }
    }

    pub fn get_mut<T: System>(&mut self) -> Result<&mut T, SystemError> {
        let system = self
            .entries
            .get_mut(&T::SYSTEM_ID)
            .ok_or_system_error(SystemError::MissingSystem(T::SYSTEM_ID))?;

        match system {
            LockedSystemEntry::Read(_, data) => Err(SystemError::MutError(data.system_id)),
            LockedSystemEntry::Write(_, data) => {
                let id = data.system_id;
                data.system
                    .downcast_mut()
                    .ok_or_system_error(SystemError::WrongSystemType(id, type_name::<T>()))
            }
        }
    }
}

impl I131 {
    fn thread_tick(
        engine: &Arc<I131>,
        engine_state: EngineState,
        systems: Vec<Arc<RwLock<SystemData>>>,
        delta: f32,
    ) -> Result<(), SystemError> {
        for system in systems {
            let mut system_data = system.write()?;

            if (engine_state == EngineState::Initialized
                || engine_state == EngineState::InEditor
                || engine_state == EngineState::Running)
                && !system_data.initialized
            {
                let context = system_data.request_context(engine.clone())?;
                system_data.system.initialize(context)?;
                system_data.initialized = true;
            }

            if engine_state == EngineState::Running && !system_data.playing {
                system_data.system.begin_play()?;
                system_data.playing = true;
            } else if engine_state != EngineState::Running && system_data.playing {
                system_data.system.end_play()?;
                system_data.playing = false;
            }

            if engine_state == EngineState::Running {
                system_data.system.update(delta)?;
            } else if engine_state == EngineState::InEditor {
                system_data.system.in_editor_update(delta)?;
            }

            if engine_state == EngineState::Stopped || system_data.queued_for_destroy {
                if system_data.playing {
                    system_data.system.end_play()?;
                    system_data.playing = false;
                }
                if system_data.initialized {
                    system_data.system.destroy()?;
                    system_data.initialized = false;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn run_thread_tick<ST: Thread131 + 'static>(
        engine: &Arc<I131>,
        engine_state: EngineState,
        thread_data: &Arc<RwLock<ThreadData>>,
    ) -> Result<EngineState, SystemError> {
        let (systems, delta, mut delta_acc) = {
            let mut thread_data = thread_data.write()?;
            let now = std::time::SystemTime::now();
            let delta_duration = now.duration_since(thread_data.last)?;
            let delta = delta_duration.as_micros() as f32
                / std::time::Duration::from_secs(1).as_micros() as f32;
            thread_data.last = now;

            let systems = thread_data
                .order
                .iter()
                .map(|id| {
                    thread_data
                        .system_data
                        .get(id)
                        .cloned()
                        .ok_or_system_error(SystemError::MissingSystem(*id))
                })
                .collect::<Result<Vec<_>, _>>()?;

            (systems, delta, thread_data.delta_acc)
        };

        match ST::TPS {
            TicksPerSecond::FullSpeed => {
                Self::thread_tick(engine, engine_state, systems, delta)?;
            }
            TicksPerSecond::Prefer(prefer) => {
                let target_delta = 1.0 / prefer;
                delta_acc += delta;
                if delta_acc >= target_delta {
                    delta_acc -= target_delta;
                    if delta_acc > target_delta {
                        eprintln!(
                            "Thread {} update is running {delta_acc} seconds behind.",
                            ST::NAME
                        );
                    }
                    Self::thread_tick(engine, engine_state, systems, prefer)?;
                } else {
                    let wait_at_least = ((target_delta - delta_acc)
                        * std::time::Duration::from_secs(1).as_millis() as f32)
                        .floor();
                    let wait = std::time::Duration::from_millis(wait_at_least as u64);
                    let millis = wait.as_millis();
                    if millis > 0 {
                        std::thread::sleep(wait);
                    }
                }
                thread_data.write()?.delta_acc = delta_acc;
            }
            TicksPerSecond::Require {
                requirement,
                threshold,
            } => {
                let target_delta = 1.0 / requirement;
                delta_acc += delta;
                if delta_acc >= target_delta {
                    delta_acc -= target_delta;
                    let behind = delta_acc > target_delta;
                    {
                        const SMALL_CHANGE: f32 = 0.0625;

                        let mut thread_data = thread_data.write()?;
                        thread_data.delta_acc = delta_acc;
                        let target = if behind { 1.0 } else { 0.0 };
                        thread_data.current_overtime_percent +=
                            (target - thread_data.current_overtime_percent) * SMALL_CHANGE;
                        if thread_data.current_overtime_percent > threshold {
                            return Err(SystemError::TpsRequirementError {
                                thread: ST::NAME,
                                requirement,
                                overtime: thread_data.current_overtime_percent * 100.0,
                            });
                        }
                    }
                    Self::thread_tick(engine, engine_state, systems, requirement)?;
                } else {
                    let wait_at_least = ((target_delta - delta_acc)
                        * std::time::Duration::from_secs(1).as_millis() as f32)
                        .floor();
                    let wait = std::time::Duration::from_millis(wait_at_least as u64);
                    std::thread::sleep(wait);
                }
                thread_data.write()?.delta_acc = delta_acc;
            }
        }

        Ok(engine.lock()?.state)
    }

    /// Contains the thread update function too
    pub(crate) fn create_thread<ST: Thread131 + 'static>(
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
            // TODO: Add custom user update functions for the thread
            let _thread = thread;

            let (thread_data, mut state) = {
                // Wait until engine is running
                let engine_data = engine.wait_until(|data| {
                    data.state == EngineState::Running && data.thread_data.contains_key(ST::NAME)
                })?;
                let thread_data = engine_data
                    .thread_data
                    .get(ST::NAME)
                    .ok_or_system_error(SystemError::SystemThreadError(format!(
                        "System thread doesn't exist for thread id {thread_id:?}"
                    )))?
                    .clone();

                {
                    let mut thread_data = thread_data.write()?;
                    thread_data.last = SystemTime::now();
                    thread_data.delta_acc = 0.0;
                    thread_data.current_overtime_percent = 0.0;
                }
                (thread_data, engine_data.state)
            };

            while state == EngineState::Running {
                state = Self::run_thread_tick::<ST>(&engine, state, &thread_data)?;
            }

            Ok(())
        };

        let join_handle = std::thread::spawn(thread_fn);

        let thread_data = Arc::new(RwLock::new(ThreadData {
            system_data: HashMap::default(),
            order: Vec::default(),
            last: SystemTime::now(),
            delta_acc: 0.0,
            join_handle: Some(join_handle),
            current_overtime_percent: 0.0,
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

        if state.system_op_queue.contains_key(&T::SYSTEM_ID)
            || state.all_systems.contains_key(&T::SYSTEM_ID)
        {
            return Err(SystemError::SystemAlreadyExists(T::SYSTEM_ID));
        }

        state
            .system_op_queue
            .insert(T::SYSTEM_ID, SystemOp::Create(system_data));

        Ok(())
    }
    pub fn destroy_system(&self, system_id: SystemId) -> Result<(), SystemError> {
        let mut state = self.lock()?;

        if !state.all_systems.contains_key(&system_id)
            || state.system_op_queue.contains_key(&system_id)
        {
            return Err(SystemError::MissingSystem(system_id));
        }

        state.system_op_queue.insert(system_id, SystemOp::Destroy);

        Ok(())
    }
    pub fn destroy_systems<I: IntoIterator<Item = SystemId>>(
        &self,
        system_ids: I,
    ) -> Result<(), SystemError> {
        let mut state = self.lock()?;

        let system_ids = system_ids
            .into_iter()
            .map(|id| (id, SystemOp::Destroy))
            .collect::<Vec<_>>();

        let any_missing_system = system_ids.iter().find(|(id, _)| {
            !state.all_systems.contains_key(id) || state.system_op_queue.contains_key(id)
        });
        if let Some((system_id, _)) = any_missing_system {
            return Err(SystemError::MissingSystem(*system_id));
        }

        state.system_op_queue.extend(system_ids);

        Ok(())
    }

    fn sort_systems(
        systems: HashMap<SystemId, RwLockReadGuard<'_, SystemData>>,
    ) -> Result<Vec<SystemId>, SystemError> {
        let mut dependencies = HashMap::<SystemId, Vec<SystemId>>::default();

        for (id, data) in systems {
            dependencies.entry(id).or_default().extend(data.before);
            for sys in data.after {
                dependencies.entry(*sys).or_default().push(id);
            }
        }

        let order = Self::create_lock_order(dependencies, true)?;

        Ok(order)
    }
    fn create_lock_order(
        dependencies: HashMap<SystemId, Vec<SystemId>>,
        allow_missing: bool,
    ) -> Result<Vec<SystemId>, SystemError> {
        let mut deps = dependencies
            .iter()
            .map(|(id, deps)| (*id, HashSet::from_iter(deps.iter().cloned())))
            .collect::<HashMap<_, HashSet<SystemId>>>();

        for (id, system_deps) in &mut deps {
            let mut missing_deps = Vec::default();
            for dep in system_deps.iter() {
                if !dependencies.contains_key(&dep) {
                    if allow_missing {
                        missing_deps.push(*dep);
                    } else {
                        return Err(SystemError::MissingDependency(*id, *dep));
                    }
                }
            }

            for dep in missing_deps {
                system_deps.remove(&dep);
            }
        }

        let mut layers = HashMap::<usize, Vec<SystemId>>::default();
        let mut placed = HashSet::<SystemId>::default();

        while !deps.is_empty() {
            let mut layer = Vec::default();

            for (id, system_deps) in &deps {
                if system_deps
                    .iter()
                    .all(|dep| !deps.contains_key(dep) && placed.contains(dep))
                {
                    layer.push(*id);
                }
            }

            if layer.is_empty() {
                return Err(SystemError::SystemCyclicDependency(
                    deps.keys().copied().collect(),
                ));
            }

            for id in &layer {
                deps.remove(id);
            }
            placed.extend(&layer);
            layers.insert(layers.len(), layer);
        }

        let mut l = layers.len() - 1;
        let mut order = Vec::default();
        while order.len() < dependencies.len() {
            order.extend_from_slice(&layers[&l]);
            if l != 0 {
                l -= 1
            };
        }

        Ok(order)
    }

    pub(crate) fn process_create_and_destroy_queues(&self) -> Result<(), SystemError> {
        let mut systems_to_destroy = vec![];
        {
            let mut state = self.lock()?;
            let queue = state.system_op_queue.drain().collect::<HashMap<_, _>>();

            let per_thread = queue.into_iter().try_fold(
                HashMap::new(),
                |mut acc, (id, op)| -> Result<_, SystemError> {
                    let thread_name = match &op {
                        SystemOp::Create(system_data) => system_data.affinity,
                        SystemOp::Destroy => state
                            .all_systems
                            .get(&id)
                            .map(|(thread, _)| thread)
                            .cloned()
                            .ok_or_system_error(SystemError::MissingSystem(id))?,
                    };
                    let entry = acc.entry(thread_name).or_insert_with(Vec::default);

                    entry.push((id, op));
                    Ok(acc)
                },
            )?;

            let mut changed = false;

            for (thread_name, ops) in per_thread {
                let thread = state
                    .get_thread_data(thread_name)
                    .cloned()
                    .ok_or_system_error(SystemError::SystemThreadError(format!(
                        "Thread {thread_name} doesn't exist"
                    )))?;
                let mut thread_data = thread.write()?;

                for (system_id, op) in ops {
                    match op {
                        SystemOp::Create(system_data) => {
                            let system_data = Arc::new(RwLock::new(system_data));
                            state
                                .all_systems
                                .insert(system_id, (thread_name, system_data.clone()));

                            thread_data.system_data.insert(system_id, system_data);
                        }
                        SystemOp::Destroy => {
                            let system = thread_data
                                .system_data
                                .remove(&system_id)
                                .ok_or_system_error(SystemError::MissingSystem(system_id))?;
                            systems_to_destroy.push(system);

                            state.all_systems.remove(&system_id);
                        }
                    }
                }

                let sorted = if thread_data.system_data.len() > 1 {
                    let unsorted = thread_data
                        .system_data
                        .iter()
                        .map(|(id, data)| Ok((*id, data.read()?)))
                        .collect::<Result<HashMap<_, _>, SystemError>>()?;
                    Self::sort_systems(unsorted)?
                } else if let Some(only_system) = thread_data.system_data.keys().next() {
                    vec![*only_system]
                } else {
                    vec![]
                };

                thread_data.order = sorted;
                changed = true;
            }

            if state.all_systems.is_empty() {
                state.state = EngineState::Stopped;
            } else if changed {
                let dependencies = state
                    .all_systems
                    .iter()
                    .map(|(id, (_, data))| Ok((*id, data.read()?.dependencies.to_vec())))
                    .collect::<Result<HashMap<SystemId, Vec<SystemId>>, SystemError>>()?;
                state.lock_order = Self::create_lock_order(dependencies, false)?;
            }
        }

        for system in systems_to_destroy {
            let mut system_data = system.write()?;

            if !system_data.destroyed {
                if system_data.playing {
                    system_data.system.end_play()?;
                    system_data.playing = false;
                }
                if system_data.initialized {
                    system_data.system.destroy()?;
                    system_data.initialized = false;
                }

                system_data.destroyed = true;
            }
        }

        Ok(())
    }
}
