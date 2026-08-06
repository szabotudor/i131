use engine131::systems::{System, SystemError, SystemId};
use engine131::{I131, MainThread, Thread131, TicksPerSecond};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

// -- Thread definitions --

struct GameThread;
impl Thread131 for GameThread {
    const NAME: &'static str = "Game";
    const TPS: TicksPerSecond = TicksPerSecond::Prefer(60.0);
    fn new() -> Self {
        Self
    }
}

// -- System definitions --

struct QuitAfter3Seconds {
    once: bool,
    acc: f32,
    update_count: Arc<AtomicU64>,
    success: Arc<AtomicBool>,
}
impl QuitAfter3Seconds {
    fn new(update_count: Arc<AtomicU64>, success: Arc<AtomicBool>) -> Self {
        Self {
            once: false,
            acc: 0.0f32,
            update_count,
            success,
        }
    }
}
impl System for QuitAfter3Seconds {
    const SYSTEM_ID: SystemId = SystemId("QuitAfter3Seconds");
    const DEPENDENCIES: &'static [SystemId] = &[GameSystemA::SYSTEM_ID, LogicSystemB::SYSTEM_ID];
    const BEFORE: &'static [SystemId] = &[GameSystemA::SYSTEM_ID];
    const AFTER: &'static [SystemId] = &[];

    fn initialize(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[QuitAfter3Seconds] initialized");
        Ok(())
    }
    fn begin_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[QuitAfter3Seconds] begin_play");
        Ok(())
    }
    fn update(&mut self, engine: &I131, _delta: f32) -> Result<(), SystemError> {
        self.acc += _delta;
        if self.acc >= 1.5 && !self.once {
            println!(
                "[QuitAfter3Seconds] 1.5 seconds elapsed, talking to GameSystemA and LogicSystemB"
            );
            self.once = true;
        }
        if self.acc >= 3.0 {
            println!("[QuitAfter3Seconds] 3 seconds elapsed, requesting shutdown");
            engine.request_immediate_shutdown()?;
            self.success.store(true, Ordering::Relaxed);
        }
        self.update_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    fn in_editor_update(&mut self, _engine: &I131, _delta: f32) -> Result<(), SystemError> {
        Ok(())
    }
    fn end_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[QuitAfter3Seconds] end_play");
        Ok(())
    }
    fn destroy(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[QuitAfter3Seconds] destroyed");
        Ok(())
    }
}

struct GameSystemA {
    update_count: Arc<AtomicU64>,
}
impl System for GameSystemA {
    const SYSTEM_ID: SystemId = SystemId("GameSystemA");
    const DEPENDENCIES: &'static [SystemId] = &[LogicSystemB::SYSTEM_ID];
    const BEFORE: &'static [SystemId] = &[];
    const AFTER: &'static [SystemId] = &[];

    fn initialize(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[GameSystemA] initialized");
        Ok(())
    }
    fn begin_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[GameSystemA] begin_play");
        Ok(())
    }
    fn update(&mut self, _engine: &I131, _delta: f32) -> Result<(), SystemError> {
        self.update_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    fn in_editor_update(&mut self, _engine: &I131, _delta: f32) -> Result<(), SystemError> {
        Ok(())
    }
    fn end_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[GameSystemA] end_play");
        Ok(())
    }
    fn destroy(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[GameSystemA] destroyed");
        Ok(())
    }
}

struct LogicSystemB {
    update_count: Arc<AtomicU64>,
}
impl System for LogicSystemB {
    const SYSTEM_ID: SystemId = SystemId("LogicSystemB");
    const DEPENDENCIES: &'static [SystemId] = &[];
    const BEFORE: &'static [SystemId] = &[LogicSystemC::SYSTEM_ID, GameSystemA::SYSTEM_ID];
    const AFTER: &'static [SystemId] = &[];

    fn initialize(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemB] initialized");
        Ok(())
    }
    fn begin_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemB] begin_play");
        Ok(())
    }
    fn update(&mut self, _engine: &I131, _delta: f32) -> Result<(), SystemError> {
        self.update_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    fn in_editor_update(&mut self, _engine: &I131, _delta: f32) -> Result<(), SystemError> {
        Ok(())
    }
    fn end_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemB] end_play");
        Ok(())
    }
    fn destroy(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemB] destroyed");
        Ok(())
    }
}

struct LogicSystemC {
    update_count: Arc<AtomicU64>,
}
impl System for LogicSystemC {
    const SYSTEM_ID: SystemId = SystemId("LogicSystemC");
    const DEPENDENCIES: &'static [SystemId] = &[LogicSystemB::SYSTEM_ID];
    const BEFORE: &'static [SystemId] = &[];
    const AFTER: &'static [SystemId] = &[LogicSystemB::SYSTEM_ID];

    fn initialize(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemC] initialized");
        Ok(())
    }
    fn begin_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemC] begin_play");
        Ok(())
    }
    fn update(&mut self, _engine: &I131, _delta: f32) -> Result<(), SystemError> {
        self.update_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    fn in_editor_update(&mut self, _engine: &I131, _delta: f32) -> Result<(), SystemError> {
        Ok(())
    }
    fn end_play(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemC] end_play");
        Ok(())
    }
    fn destroy(&mut self, _engine: &I131) -> Result<(), SystemError> {
        println!("[LogicSystemC] destroyed");
        Ok(())
    }
}

#[test]
fn stress_two_threads_four_systems_quit_after_3s() {
    let quit_updates = Arc::new(AtomicU64::new(0));
    let game_updates = Arc::new(AtomicU64::new(0));
    let logic_b_updates = Arc::new(AtomicU64::new(0));
    let logic_c_updates = Arc::new(AtomicU64::new(0));

    let success = Arc::new(AtomicBool::new(false));

    let engine = I131::new().expect("failed to create engine");

    // GameThread (FullSpeed) — 2 systems
    engine
        .create_system(
            QuitAfter3Seconds::new(quit_updates.clone(), success.clone()),
            MainThread::AFFINITY,
        )
        .expect("failed to create QuitAfter3Seconds on GameThread");
    engine
        .create_system(
            GameSystemA {
                update_count: Arc::clone(&game_updates),
            },
            MainThread::AFFINITY,
        )
        .expect("failed to create GameSystemA on GameThread");

    // LogicThread (60 FPS) — 2 systems
    engine
        .create_system(
            LogicSystemB {
                update_count: Arc::clone(&logic_b_updates),
            },
            GameThread::AFFINITY,
        )
        .expect("failed to create LogicSystemB on LogicThread");
    engine
        .create_system(
            LogicSystemC {
                update_count: Arc::clone(&logic_c_updates),
            },
            GameThread::AFFINITY,
        )
        .expect("failed to create LogicSystemC on LogicThread");

    // Watchdog — abort if test hangs
    let engine_for_watchdog = Arc::clone(&engine);
    let watchdog = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(4));
        if success.load(Ordering::Relaxed) {
            return;
        }
        eprintln!("[watchdog] test hung for 10s, aborting");
        engine_for_watchdog.request_immediate_shutdown().ok();
    });

    // Run — QuitAfter3Seconds will call request_immediate_shutdown after 3s
    let start = Instant::now();
    engine.main_loop().expect("main_loop returned error");
    let elapsed = start.elapsed();

    watchdog.join().expect("watchdog panicked");

    println!(
        "test completed in {:.2}s| QuitAfter3Seconds updates: {} | GameSystemA updates: {} | LogicSystemB updates: {} | LogicSystemC updates: {}",
        elapsed.as_secs_f64(),
        quit_updates.load(Ordering::Relaxed),
        game_updates.load(Ordering::Relaxed),
        logic_b_updates.load(Ordering::Relaxed),
        logic_c_updates.load(Ordering::Relaxed),
    );

    assert!(
        elapsed >= Duration::from_secs(3),
        "engine shut down too early: {:?}",
        elapsed
    );
    assert!(
        elapsed < Duration::from_secs(10),
        "engine did not shut down in time (watchdog should have fired): {:?}",
        elapsed
    );
}
