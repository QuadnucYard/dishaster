use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use dishaster_core::{
    Tick,
    commands::SimCommand,
    sim::Simulation,
    snapshots::{PresentationEvent, Snapshot},
};
use fibre::spsc;

/// A snapshot frame paired with the number of simulation ticks it represents.
/// This is the unit sent over the channel from the sim thread to the main thread.
pub struct SnapshotFrame {
    pub ticks: Tick,
    pub snapshot: Snapshot,
    pub events: Vec<PresentationEvent>,
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
    paused: Arc<AtomicBool>,
    pause_condvar: Arc<(Mutex<bool>, Condvar)>,
    handle: Option<JoinHandle<()>>,
}

impl SimController {
    /// Stop the sim thread and join handle.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // Wake up the thread if it's paused
        self.resume();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// Pause the simulation thread.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    /// Resume the simulation thread.
    pub fn resume(&self) {
        let (lock, condvar) = &*self.pause_condvar;
        let mut paused = lock.lock().unwrap();
        *paused = false;
        condvar.notify_one();
    }

    /// Check if simulation is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}

impl Drop for SimController {
    fn drop(&mut self) {
        // ensure thread is stopped
        self.stop();
    }
}

pub struct SnapshotReceiver(spsc::BoundedSyncReceiver<SnapshotFrame>);

impl SimulationRunner {
    pub fn new(mut sim: Simulation, tps: f64) -> Self {
        // create channel
        let (tx, rx) = spsc::bounded_sync::<SnapshotFrame>(3);

        // stop flag
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();

        // pause flag and condition variable
        let paused = Arc::new(AtomicBool::new(false));
        let paused_clone = paused.clone();
        let pause_condvar = Arc::new((Mutex::new(false), Condvar::new()));
        let pause_condvar_clone = pause_condvar.clone();

        // spawn sim thread
        let handle = thread::spawn(move || {
            let dt = 1.0 / tps;
            let mut last = Instant::now();

            while !stop_clone.load(Ordering::Relaxed) {
                // Check if paused and wait for resume
                if paused_clone.load(Ordering::Relaxed) {
                    let (lock, condvar) = &*pause_condvar_clone;
                    let mut paused_flag = lock.lock().unwrap();
                    while *paused_flag {
                        paused_flag = condvar.wait(paused_flag).unwrap();
                    }
                }

                let now = Instant::now();
                if now.duration_since(last).as_secs_f64() >= dt {
                    last = now;
                    sim.tick();
                    let _ = tx.send(SnapshotFrame {
                        ticks: 1, // to be updated by receiver
                        snapshot: sim.snapshot(),
                        events: sim.poll_events(),
                    });
                }
                thread::sleep(Duration::from_millis(1));
            }
            // thread exiting
        });

        Self {
            snapshot_receiver: SnapshotReceiver(rx),
            sim_ctrl: SimController {
                stop,
                paused,
                pause_condvar,
                handle: Some(handle),
            },
        }
    }

    pub fn poll_snapshot(&mut self) -> Option<SnapshotFrame> {
        let mut last_snap = None;
        while let Ok(snap) = self.snapshot_receiver.0.try_recv() {
            last_snap = Some(snap);
        }
        last_snap
    }

    /// Pause the simulation.
    pub fn pause(&self) {
        self.sim_ctrl.pause();
    }

    /// Resume the simulation.
    pub fn resume(&self) {
        self.sim_ctrl.resume();
    }

    /// Check if simulation is currently paused.
    pub fn is_paused(&self) -> bool {
        self.sim_ctrl.is_paused()
    }
}

/// Synchronous simulation runner that advances on demand.
/// Useful for testing, debugging, or when you need precise control over simulation timing.
///
/// Unlike `SimulationRunner` which runs asynchronously in a background thread,
/// this runner gives you direct control over when and how much the simulation advances.
pub struct SyncSimulationRunner {
    sim: Simulation,
    accumulator: f64,
    tps: f64,
}

impl SyncSimulationRunner {
    /// Create a new synchronous simulation runner.
    pub fn new(sim: Simulation, tps: f64) -> Self {
        Self {
            sim,
            accumulator: 0.0,
            tps,
        }
    }

    /// Advance the simulation by the given delta time.
    /// Returns the snapshot if the simulation was advanced (i.e., accumulator reached the tick threshold).
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds to advance the simulation
    ///
    /// # Returns
    /// * `Some(Snapshot)` if the simulation ticked and produced a new snapshot
    /// * `None` if the accumulator hasn't reached the tick threshold yet
    pub fn tick(&mut self, dt: f64) -> Option<SnapshotFrame> {
        self.accumulator += dt;

        let mut result = None;
        let mut ticks = 0;

        loop {
            let fixed_dt = 1.0 / self.tps.max(1.0);
            if self.accumulator < fixed_dt {
                break;
            }
            // Advance simulation by one fixed timestep
            self.sim.tick();
            ticks += 1;

            // Subtract fixed dt, keeping any remainder for next frame
            self.accumulator -= fixed_dt;

            result = Some(SnapshotFrame {
                ticks,
                snapshot: self.sim.snapshot(),
                events: self.sim.poll_events(),
            });
        }

        result
    }

    /// Force advance the simulation by one tick, regardless of accumulator.
    /// This is useful for testing or when you need immediate advancement.
    ///
    /// # Returns
    /// The new snapshot after the forced tick
    pub fn force_tick(&mut self) -> Snapshot {
        self.sim.tick();

        // Reset accumulator since we forced a tick
        self.accumulator = 0.0;

        self.sim.snapshot()
    }

    /// Forward a simulation command to the underlying simulation immediately.
    pub fn send_command(&mut self, command: SimCommand) {
        // todo: commands may be queued and handled in batch
        self.sim.handle_command(command);
    }

    /// Update the target ticks-per-second rate used for fixed-step advancement.
    pub fn set_tps(&mut self, tps: f64) {
        if (self.tps - tps).abs() > f64::EPSILON {
            self.tps = tps;
        }
    }

    /// Retrieve the current ticks-per-second target.
    pub fn tps(&self) -> f64 {
        self.tps
    }
}
