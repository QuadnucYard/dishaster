//! Integration tests for Dishaster.

mod persist;

use std::sync::Arc;

use anyhow::Result;
use dishaster_core::{
    interface::{SimCommand, SimEvent},
    models::Day,
    sim::{ISimulation, Simulation},
};
use dishaster_data::DataLoader;
use dishaster_persistence::{PersistentStorage, UserDataService};
use dishrupt_rng::{Prng, prelude::Rng};

use crate::persist::{level_for_current_day, save_sim_profile};

/// In-memory persistent storage for testing purposes.
pub struct MemoryStorage;

impl PersistentStorage for MemoryStorage {
    fn read(&self, _path: &str) -> Result<Option<Vec<u8>>> {
        Ok(None) // Always return None (no data)
    }

    fn write_atomic(&self, _path: &str, _bytes: &[u8]) -> Result<()> {
        Ok(()) // Do nothing on write
    }

    fn delete(&self, _path: &str) -> Result<()> {
        Ok(()) // Do nothing on delete
    }
}

/// Extension trait to run multiple simulation steps.
trait RunSteps {
    fn run_steps(&mut self, steps: u32);

    fn run_steps_with_monitor(
        &mut self,
        steps: u32,
        monitor: impl FnMut(SimEvent, &mut Simulation),
    );
}

impl RunSteps for Simulation {
    fn run_steps(&mut self, steps: u32) {
        for i in 0..steps {
            if i > 0 && i % 1000 == 0 {
                println!("=== Step {} ===", i);
            }
            self.tick();
        }
    }

    fn run_steps_with_monitor(
        &mut self,
        steps: u32,
        mut monitor: impl FnMut(SimEvent, &mut Simulation),
    ) {
        for i in 0..steps {
            if i > 0 && i % 1000 == 0 {
                println!("=== Step {} ===", i);
            }
            self.tick();
            let events = self.poll_events();
            for event in events {
                monitor(event, self);
            }
        }
    }
}

fn main() -> Result<()> {
    env_logger::Builder::new().init();

    let mut loader = DataLoader::new("../assets/data")?;
    let data = loader.load_all_data()?;
    let registry = data.models;

    dishaster_validation::validate_registry(&registry)?;
    println!("✓ Data validation passed");

    let service = UserDataService::new(Arc::new(MemoryStorage));
    let profile_svc = &service.profiles;
    dishaster_validation::validate_player_profile(&profile_svc.load().unwrap(), &registry)?;
    println!("✓ Player profile validation passed");

    let level = level_for_current_day(profile_svc, &registry)?;
    let start_day = level.day;

    let mut sim = Simulation::new(registry.clone());
    sim.start(level);

    // Let the simulation stabilize a bit
    sim.run_steps(100);

    // Start the run
    sim.command(SimCommand::StartRun);

    // Run with event monitoring to refill empty dispensers
    sim.run_steps_with_monitor(20000, |event, sim| {
        if let SimEvent::DispenserStockChanged {
            entity,
            current_stock,
            ..
        } = event
            && current_stock == 0
        {
            println!("Dispenser {:?} is empty, sending refill command", entity);
            sim.command(SimCommand::RefillDispenser(entity));
        }
    });

    // End the run and check for completion events
    sim.command(SimCommand::EndRun);
    sim.run_steps(1);
    let events = sim.poll_events();
    println!("Events: {}", events.len());
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
    println!("Events after decision: {}", events.len());
    assert!(events.iter().any(|e| matches!(e, SimEvent::Persist)));
    assert!(events.iter().any(|e| matches!(e, SimEvent::DayCompleted)));

    let persisted = sim.persist();
    println!("Persisted profile");
    assert_eq!(persisted.current_day, Day(start_day.0 + 1)); // Day should have advanced

    // Print first day stats
    let day_stats = &persisted.day_stats;
    println!("Day 0 stats:");
    println!(
        "  Consumption: {:.2} kg, Revenue: ¥{:.2}, Completed: {} / {}",
        day_stats.consumption_kg,
        day_stats.revenue,
        day_stats.completed_diners,
        day_stats.total_visits
    );

    save_sim_profile(profile_svc, persisted)?;
    std::mem::drop(sim);

    // now run more days
    let mut rng = Prng::new(42);
    for i in 0..100 {
        println!("--- Day {} ---", i + 1);

        // Start next day
        let level = level_for_current_day(profile_svc, &registry)?;
        let mut sim = Simulation::new(registry.clone());
        sim.start(level);
        sim.run_steps(10);

        // Start the run
        sim.command(SimCommand::StartRun);
        sim.run_steps(100);

        // End the run
        sim.command(SimCommand::EndRun);
        sim.run_steps(1);

        let events = sim.poll_events();
        assert!(events.iter().any(|e| matches!(e, SimEvent::RunCompleted)));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_)))
        );

        // Apply a management decision
        sim.command(SimCommand::ApplyManagementDecision(rng.random_range(0..3)));
        sim.run_steps(1);

        let persisted = sim.persist();
        save_sim_profile(profile_svc, persisted)?;
    }

    // Print aggregate stats at the end
    let profile = profile_svc.load().unwrap();
    println!("\n=== Final Statistics ===");
    println!(
        "Total consumption: {:.2} kg",
        profile.aggregates.lifetime_consumption_kg
    );
    println!("Total revenue: ¥{:.2}", profile.aggregates.lifetime_revenue);
    println!(
        "Total served: {} diners",
        profile.aggregates.lifetime_served
    );
    println!("Days of history: {}", profile.daily_history.len());

    // Show last 5 days stats
    println!("\nLast 5 days:");
    for day_stats in profile.daily_history.iter().rev().take(5).rev() {
        println!(
            "  Day {}: {:.2}kg, ¥{:.0}, {}/{} completed",
            day_stats.day.0,
            day_stats.consumption_kg,
            day_stats.revenue,
            day_stats.completed_diners,
            day_stats.total_visits
        );
    }

    println!("\nTest completed successfully.");

    Ok(())
}
