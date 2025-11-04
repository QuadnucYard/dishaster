use dishrupt_simulation::{ISimulation, SimulationFeature};

use crate::{SimulationRunner, SnapshotFrame, extend_frame};

/// Synchronous simulation runner that advances on demand.
/// Useful for testing, debugging, or when you need precise control over simulation timing.
///
/// Unlike `SimulationRunner` which runs asynchronously in a background thread,
/// this runner gives you direct control over when and how much the simulation advances.
pub struct SyncSimulationRunner<F: SimulationFeature> {
    sim: Box<dyn ISimulation<F>>,
    accumulator: f64,
    tps: f64,
}

impl<F: SimulationFeature> SyncSimulationRunner<F> {
    /// Create a new synchronous simulation runner.
    pub fn new(sim: Box<dyn ISimulation<F>>, tps: f64) -> Self {
        Self {
            sim,
            accumulator: 0.0,
            tps,
        }
    }
}

impl<F: SimulationFeature> SimulationRunner<F> for SyncSimulationRunner<F> {
    fn tick(&mut self, dt: f64) -> Option<SnapshotFrame<F>> {
        self.accumulator += dt;

        let mut result: Option<SnapshotFrame<F>> = None;
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

            extend_frame(
                &mut result,
                SnapshotFrame {
                    ticks,
                    snapshot: self.sim.snapshot(),
                    events: self.sim.poll_events(),
                    responses: self.sim.poll_responses(),
                },
            );
        }

        result
    }

    fn force_tick(&mut self) -> F::Snapshot {
        self.sim.tick();

        // Reset accumulator since we forced a tick
        self.accumulator = 0.0;

        self.sim.snapshot()
    }

    fn send_command(&mut self, command: F::Command) {
        // todo: commands may be queued and handled in batch
        self.sim.command(command);
    }

    fn send_query(&mut self, query: F::Query) {
        self.sim.query(query);
    }

    fn set_tps(&mut self, tps: f64) {
        if (self.tps - tps).abs() > f64::EPSILON {
            self.tps = tps;
        }
    }

    fn tps(&self) -> f64 {
        self.tps
    }
}
