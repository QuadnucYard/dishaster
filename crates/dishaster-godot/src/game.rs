use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use dishaster_core::{
    models::LevelConfig,
    sim::{Simulation, Snapshot},
};
use dishrupt_godot::input::listener::GodotInputEvent;
use fibre::spsc;
use godot::{classes::Node, obj::Gd};

use crate::game_main::GAME_DATA;

pub struct Game {
    sim_runner: SimulationRunner,
}

impl Game {
    pub fn new(_gd: Gd<Node>, level: LevelConfig) -> Self {
        let mut sim = Simulation::new(GAME_DATA.get().unwrap().clone());
        sim.start(level);
        let sim_runner = SimulationRunner::new(sim);
        Self { sim_runner }
    }

    pub fn process(&mut self) {
        if let Some(_snapshot) = self.sim_runner.poll_snapshot() {
            // process snapshot
        }
    }

    pub fn process_input(&mut self, _event: GodotInputEvent) {
        // handle input event
    }
}

pub struct SimulationRunner {
    snapshot_receiver: SnapshotReceiver,
    #[allow(unused)]
    sim_ctrl: SimController,
}

/// Controller resource that owns the sim thread handle and a stop flag.
/// On exiting level we set `stop` and join the handle to cleanly terminate thread.
pub struct SimController {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl SimController {
    /// Stop the sim thread and join handle.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for SimController {
    fn drop(&mut self) {
        // ensure thread is stopped
        self.stop();
    }
}

pub struct SnapshotReceiver(spsc::BoundedSyncReceiver<Snapshot>);

impl SimulationRunner {
    pub fn new(mut sim: Simulation) -> Self {
        // create channel
        let (tx, rx) = spsc::bounded_sync::<Snapshot>(3);

        // stop flag
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();

        // spawn sim thread
        let handle = thread::spawn(move || {
            let tick_hz = 60.0_f64;
            let dt = 1.0 / tick_hz;
            let mut last = Instant::now();

            while !stop_clone.load(Ordering::Relaxed) {
                let now = Instant::now();
                if now.duration_since(last).as_secs_f64() >= dt {
                    last = now;
                    sim.tick(dt);
                    let snap = sim.snapshot();
                    let _ = tx.send(snap);
                }
                thread::sleep(Duration::from_millis(1));
            }
            // thread exiting
        });

        Self {
            snapshot_receiver: SnapshotReceiver(rx),
            sim_ctrl: SimController {
                stop,
                handle: Some(handle),
            },
        }
    }

    pub fn poll_snapshot(&mut self) -> Option<Snapshot> {
        let mut last_snap = None;
        while let Ok(snap) = self.snapshot_receiver.0.try_recv() {
            last_snap = Some(snap);
        }
        last_snap
    }
}
