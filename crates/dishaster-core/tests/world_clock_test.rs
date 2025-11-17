//! Test world clock system to verify proper time display and freezing

mod fixture;

use dishaster_core::sim::*;
use dishrupt_simulation::ISimulation;
use fixture::{create_test_level, create_test_registry};

#[test]
fn test_world_clock_starts_at_entry_time() {
    let registry = create_test_registry();
    let level = create_test_level();

    let mut sim = Simulation::new(registry);
    sim.start(level);

    let snapshot = sim.snapshot();

    // World time should start at entry_time 11:00:00 (39600 seconds)
    assert_eq!(
        snapshot.stats.world_time, 39600.0,
        "World clock should start at configured entry_time (11:00:00)"
    );

    // Simulation time should start at 0
    assert_eq!(
        snapshot.stats.time_seconds, 0.0,
        "Simulation time should start at 0"
    );
}

#[test]
fn test_time_advances_normally_before_run() {
    let registry = create_test_registry();
    let level = create_test_level();

    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Tick a few times before starting the run
    for _ in 0..10 {
        sim.tick();
    }

    let snapshot = sim.snapshot();

    // Tick counter should advance
    assert_eq!(snapshot.stats.tick, 10, "Tick counter should advance");

    // World time should advance from entry_time
    assert!(
        snapshot.stats.world_time > 39600.0,
        "World time should advance from entry_time during preparation"
    );

    // Simulation time should advance
    assert!(
        snapshot.stats.time_seconds > 0.0,
        "Simulation time should advance"
    );
}

#[test]
fn test_fast_forward_on_run_start() {
    let registry = create_test_registry();
    let level = create_test_level();

    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Time starts at entry_time (11:00:00 = 39600)
    let snapshot_before = sim.snapshot();
    assert_eq!(snapshot_before.stats.world_time, 39600.0);

    // Send start run command - should fast-forward to start_time (11:30:00 = 41400)
    use dishaster_core::interface::SimCommand;
    sim.command(SimCommand::StartRun);

    let snapshot_after = sim.snapshot();

    // World time should have jumped to start_time
    assert!(
        snapshot_after.stats.world_time >= 41400.0,
        "World time should fast-forward to start_time (11:30:00): got {}",
        snapshot_after.stats.world_time
    );

    // Tick a few more times
    for _ in 0..10 {
        sim.tick();
    }

    let snapshot_final = sim.snapshot();

    // Time should continue advancing normally after fast-forward
    assert!(
        snapshot_final.stats.world_time > snapshot_after.stats.world_time,
        "Time should continue advancing after fast-forward"
    );
}
