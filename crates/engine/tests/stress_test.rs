use std::time::{Duration, SystemTime};

use engine131::{
    schedulers::DAGScheduler,
    systems::{System, SystemError, SystemId},
    I131,
};

fn spin(n: u64) {
    let mut x = n | 1;
    for i in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(i);
    }
    std::hint::black_box(x);
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

// sys0: no deps, counts frames
struct Sys0 {
    tick: u64,
    rng: u64,
}

impl Sys0 {
    fn new() -> Self {
        Self { tick: 0, rng: 0xdeadbeef_cafebabe }
    }
}

impl System for Sys0 {
    fn system_id() -> SystemId where Self: Sized { SystemId("sys0") }
    fn dependencies() -> &'static [SystemId] where Self: Sized { &[] }
    fn initialize(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn begin_play(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn update(&mut self, _: &I131, _: f32) -> Result<(), SystemError> {
        let n = (xorshift(&mut self.rng) % 300 + 50) * 1000;
        spin(n);
        self.tick += 1;
        Ok(())
    }
    fn in_editor_update(&mut self, e: &I131, d: f32) -> Result<(), SystemError> { self.update(e, d) }
    fn end_play(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn destroy(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
}

// sys1: deps=[sys0], must read sys0's current-frame tick
struct Sys1 {
    tick: u64,
    rng: u64,
}

impl Sys1 {
    fn new() -> Self {
        Self { tick: 0, rng: 0xc0ffee_facade }
    }
}

impl System for Sys1 {
    fn system_id() -> SystemId where Self: Sized { SystemId("sys1") }
    fn dependencies() -> &'static [SystemId] where Self: Sized { &[SystemId("sys0")] }
    fn initialize(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn begin_play(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn update(&mut self, engine: &I131, _: f32) -> Result<(), SystemError> {
        self.tick += 1;

        let sys0 = engine.system::<Sys0>(&SystemId("sys0"))?;
        let sys0_tick = sys0.read::<Sys0>()?.tick;

        // sys0 must have already run this frame: its tick must equal ours
        assert_eq!(
            sys0_tick, self.tick,
            "sys1 frame {}: sys0.tick={} (expected {})",
            self.tick, sys0_tick, self.tick
        );

        let n = (xorshift(&mut self.rng) % 300 + 50) * 1000;
        spin(n);
        Ok(())
    }
    fn in_editor_update(&mut self, e: &I131, d: f32) -> Result<(), SystemError> { self.update(e, d) }
    fn end_play(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn destroy(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
}

// sys2: deps=[sys0, sys1], must read both current-frame, destroys all after 10s
struct Sys2 {
    tick: u64,
    rng: u64,
    start: SystemTime,
    done: bool,
}

impl Sys2 {
    fn new() -> Self {
        Self { tick: 0, rng: 0xabad1dea_deadc0de, start: SystemTime::UNIX_EPOCH, done: false }
    }
}

impl System for Sys2 {
    fn system_id() -> SystemId where Self: Sized { SystemId("sys2") }
    fn dependencies() -> &'static [SystemId] where Self: Sized {
        &[SystemId("sys0"), SystemId("sys1")]
    }
    fn initialize(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn begin_play(&mut self, _: &I131) -> Result<(), SystemError> {
        self.start = SystemTime::now();
        Ok(())
    }
    fn update(&mut self, engine: &I131, _: f32) -> Result<(), SystemError> {
        self.tick += 1;

        let sys0 = engine.system::<Sys0>(&SystemId("sys0"))?;
        let sys0_tick = sys0.read::<Sys0>()?.tick;
        let sys1 = engine.system::<Sys1>(&SystemId("sys1"))?;
        let sys1_tick = sys1.read::<Sys1>()?.tick;

        assert_eq!(
            sys0_tick, self.tick,
            "sys2 frame {}: sys0.tick={} (expected {})",
            self.tick, sys0_tick, self.tick
        );
        assert_eq!(
            sys1_tick, self.tick,
            "sys2 frame {}: sys1.tick={} (expected {})",
            self.tick, sys1_tick, self.tick
        );

        let n = (xorshift(&mut self.rng) % 300 + 50) * 1000;
        spin(n);

        if !self.done && self.start.elapsed().unwrap_or(Duration::ZERO) >= Duration::from_secs(10) {
            self.done = true;
            engine.destroy_system(SystemId("sys2"))?;
            engine.destroy_system(SystemId("sys1"))?;
            engine.destroy_system(SystemId("sys0"))?;
        }

        Ok(())
    }
    fn in_editor_update(&mut self, e: &I131, d: f32) -> Result<(), SystemError> { self.update(e, d) }
    fn end_play(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
    fn destroy(&mut self, _: &I131) -> Result<(), SystemError> { Ok(()) }
}

#[test]
fn stress_cross_thread_deps() {
    let engine = I131::new(2, Box::new(DAGScheduler::new())).unwrap();
    engine.initialize().unwrap();
    engine.create_system(Sys0::new()).unwrap();
    engine.create_system(Sys1::new()).unwrap();
    engine.create_system(Sys2::new()).unwrap();
    engine.main_loop().unwrap();
}
