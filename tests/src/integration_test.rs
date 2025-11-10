//! Integration tests for Dishaster.

use std::sync::Arc;

use anyhow::Result;
use dishaster_core::{
    interface::{SimCommand, SimEvent},
    models::Day,
    sim::{ISimulation, Simulation},
};
use dishaster_data::DataLoader;
use dishaster_persistence::{PersistentStorage, Persister, PlayerService};

/// In-memory persistent storage for testing purposes.
pub struct MemoryStorage;

impl PersistentStorage for MemoryStorage {
    fn load_or_create_with<T, P: Persister<T>>(
        &mut self,
        _path: &str,
        init: impl FnOnce() -> T,
    ) -> Result<T> {
        // Always return a new instance
        Ok(init())
    }

    fn save_with<T, P: Persister<T>>(&mut self, _path: &str, _data: &T) -> Result<()> {
        // Saves nothing
        Ok(())
    }
}

/// Extension trait to run multiple simulation steps.
trait RunSteps {
    fn run_steps(&mut self, steps: u32);
}

impl RunSteps for Simulation {
    fn run_steps(&mut self, steps: u32) {
        for _ in 0..steps {
            self.tick();
        }
    }
}

fn main() -> Result<()> {
    let loader = DataLoader::new("../assets/data")?;
    let data = loader.load_all_data()?;
    let registry = Arc::new(data.models);

    let service = PlayerService::load_or_create(MemoryStorage, registry.clone(), None)?;
    let level = service.level_for_current_day()?;
    let start_day = level.day;

    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Let the simulation stabilize a bit
    sim.run_steps(100);

    // Start the run
    sim.command(SimCommand::StartRun);
    sim.run_steps(100);

    // End the run and check for completion events
    sim.command(SimCommand::EndRun);
    sim.run_steps(1);
    let events = sim.poll_events();
    println!("Events: {} {:#?}", events.len(), events);
    assert!(events.iter().any(|e| matches!(e, SimEvent::RunCompleted)));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_)))
    );

    // Apply a management decision (index 0 for test)
    sim.command(SimCommand::ApplyManagementDecision(0));
    sim.run_steps(1);
    let events = sim.poll_events();
    println!("Events after decision: {} {:#?}", events.len(), events);
    assert!(events.iter().any(|e| matches!(e, SimEvent::Persist)));
    assert!(events.iter().any(|e| matches!(e, SimEvent::DayCompleted)));

    let persisted = sim.persist();
    println!("Persisted profile");
    assert_eq!(persisted.current_day, Day(start_day.0 + 1)); // Day should have advanced

    println!("Test completed successfully.");

    Ok(())
}
