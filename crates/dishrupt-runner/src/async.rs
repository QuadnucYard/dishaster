use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use dishrupt_simulation::{ISimulation, SimulationFeature};
use fibre::spsc;

use crate::{SimulationRunner, SnapshotFrame, extend_frame};

/// Asynchronous simulation runner that runs the simulation in a background thread.
pub struct AsyncSimulationRunner<F: SimulationFeature> {
    snapshot_receiver: SnapshotReceiver<F>,
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

struct SnapshotReceiver<F: SimulationFeature>(spsc::BoundedSyncReceiver<SnapshotFrame<F>>);

impl<F: SimulationFeature + 'static> AsyncSimulationRunner<F>
where
    SnapshotFrame<F>: Send,
{
    /// Create a new asynchronous simulation runner.
    pub fn new(mut sim: Box<dyn ISimulation<F> + Send>, tps: f64) -> Self {
        // create channel
        let (tx, rx) = spsc::bounded_sync(3);

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
            let mut tick = 0;
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
                    tick += 1;
                    let _ = tx.send(SnapshotFrame {
                        tick,
                        delta_ticks: 1, // to be updated by receiver
                        snapshot: sim.snapshot(),
                        events: sim.poll_events(),
                        responses: sim.poll_responses(),
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

    /// Poll for the latest snapshot frame from the simulation thread.
    pub fn poll_snapshot(&mut self) -> Option<SnapshotFrame<F>> {
        let mut last_snap: Option<SnapshotFrame<F>> = None;
        while let Ok(snap) = self.snapshot_receiver.0.try_recv() {
            extend_frame(&mut last_snap, snap);
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

#[allow(unused)]
impl<F: SimulationFeature + 'static> SimulationRunner<F> for AsyncSimulationRunner<F>
where
    SnapshotFrame<F>: Send,
{
    fn tick(&mut self, dt: f64) -> Option<SnapshotFrame<F>> {
        self.poll_snapshot()
    }

    fn force_tick(&mut self) -> F::Snapshot {
        todo!()
    }

    fn send_command(&mut self, command: F::Command) {
        todo!()
    }

    fn send_query(&mut self, query: F::Query) {
        todo!()
    }

    fn tps(&self) -> f64 {
        todo!()
    }

    fn set_tps(&mut self, tps: f64) {
        todo!()
    }
}
