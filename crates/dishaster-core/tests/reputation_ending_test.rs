//! Integration test for reputation-based endings
//!
//! Tests that good and bad reputation endings can be triggered correctly
//! by adjusting reputation and ending the run.

mod fixture;

use dishaster_core::sim::Simulation;
use dishaster_interface::{SimCommand, event::SimEvent};
use dishaster_views::EndingType;
use dishrupt_simulation::ISimulation;
use fixture::{create_test_level, create_test_registry};

/// Helper to find ShowEnding event in polled events
fn find_ending_event(events: &[SimEvent]) -> Option<EndingType> {
    events.iter().find_map(|e| {
        if let SimEvent::ShowEnding(view) = e {
            match view.id.as_str() {
                "bad_reputation" => Some(EndingType::BadReputation),
                "good_reputation" => Some(EndingType::GoodReputation),
                "rectification" => Some(EndingType::Rectification),
                _ => None,
            }
        } else {
            None
        }
    })
}

#[test]
fn test_bad_reputation_ending_triggered() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    println!("=== Testing Bad Reputation Ending ===");

    // Start the run to transition to Running phase
    sim.command(SimCommand::StartRun);
    sim.tick();
    sim.poll_events(); // Clear initial events

    println!("Run started, adjusting reputation to 0...");

    // Adjust reputation to 0 (this should trigger bad ending when run ends)
    sim.command(SimCommand::DevAdjustReputation(-100.0));
    sim.tick();

    // Poll events to see reputation update
    let events = sim.poll_events();
    println!(
        "Events after reputation adjustment: {} events",
        events.len()
    );
    for event in &events {
        if let SimEvent::ReputationUpdate(rep) = event {
            println!(
                "  - Reputation updated: {:.2} (delta: {:.2})",
                rep.reputation, rep.reputation_delta
            );
            assert!(
                rep.reputation <= 0.01,
                "Reputation should be at or near 0, got {:.2}",
                rep.reputation
            );
        }
    }

    // Manually end the run
    println!("Ending run...");
    sim.command(SimCommand::EndRun);
    sim.tick();

    let events = sim.poll_events();
    println!("Events after ending run: {} events", events.len());

    // Check for ShowManagementDecisions event
    let has_decisions = events
        .iter()
        .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_)));
    println!("Management decisions rolled: {}", has_decisions);

    // Now trigger management decision to advance to next day
    // (This simulates player selecting a decision in settlement phase)
    println!("Applying management decision to advance day...");
    sim.command(SimCommand::ApplyManagementDecision(0));
    sim.tick();

    // Poll events to check for ending (should happen in AdvanceDay)
    let events = sim.poll_events();
    println!("Events after advancing day: {} events", events.len());
    for event in &events {
        match event {
            SimEvent::ShowEnding(view) => {
                println!(
                    "  - ShowEnding event: id={}, can_continue={}",
                    view.id, view.can_continue
                );
            }
            SimEvent::DayCompleted => {
                println!("  - DayCompleted event");
            }
            SimEvent::Persist => {
                println!("  - Persist event");
            }
            _ => {}
        }
    }

    // Check that bad reputation ending was triggered
    let ending = find_ending_event(&events);
    assert!(
        ending.is_some(),
        "Expected ending event to be emitted, but none found"
    );
    assert_eq!(
        ending.unwrap(),
        EndingType::BadReputation,
        "Expected BadReputation ending when reputation is 0"
    );

    println!("✓ Bad reputation ending triggered successfully");
}

#[test]
fn test_good_reputation_ending_triggered() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    println!("=== Testing Good Reputation Ending ===");

    // Start the run to transition to Running phase
    sim.command(SimCommand::StartRun);
    sim.tick();
    sim.poll_events(); // Clear initial events

    println!("Run started, adjusting reputation to 100...");

    // Adjust reputation to 100 (this should trigger good ending when run ends)
    sim.command(SimCommand::DevAdjustReputation(100.0));
    sim.tick();

    // Poll events to see reputation update
    let events = sim.poll_events();
    println!(
        "Events after reputation adjustment: {} events",
        events.len()
    );
    for event in &events {
        if let SimEvent::ReputationUpdate(rep) = event {
            println!(
                "  - Reputation updated: {:.2} (delta: {:.2})",
                rep.reputation, rep.reputation_delta
            );
            assert!(
                rep.reputation >= 99.99,
                "Reputation should be at or near 100, got {:.2}",
                rep.reputation
            );
        }
    }

    // Manually end the run
    println!("Ending run...");
    sim.command(SimCommand::EndRun);
    sim.tick();

    let events = sim.poll_events();
    println!("Events after ending run: {} events", events.len());

    // Check for ShowManagementDecisions event
    let has_decisions = events
        .iter()
        .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_)));
    println!("Management decisions rolled: {}", has_decisions);

    // Now trigger management decision to advance to next day
    // (This simulates player selecting a decision in settlement phase)
    println!("Applying management decision to advance day...");
    sim.command(SimCommand::ApplyManagementDecision(0));
    sim.tick();

    // Poll events to check for ending (should happen in AdvanceDay)
    let events = sim.poll_events();
    println!("Events after advancing day: {} events", events.len());
    for event in &events {
        match event {
            SimEvent::ShowEnding(view) => {
                println!(
                    "  - ShowEnding event: id={}, can_continue={}",
                    view.id, view.can_continue
                );
            }
            SimEvent::DayCompleted => {
                println!("  - DayCompleted event");
            }
            SimEvent::Persist => {
                println!("  - Persist event");
            }
            _ => {}
        }
    }

    // Check that good reputation ending was triggered
    let ending = find_ending_event(&events);
    assert!(
        ending.is_some(),
        "Expected ending event to be emitted, but none found"
    );
    assert_eq!(
        ending.unwrap(),
        EndingType::GoodReputation,
        "Expected GoodReputation ending when reputation is 100"
    );

    println!("✓ Good reputation ending triggered successfully");
}

#[test]
fn test_no_ending_with_moderate_reputation() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    println!("=== Testing No Ending with Moderate Reputation ===");

    // Start the run
    sim.command(SimCommand::StartRun);
    sim.tick();
    sim.poll_events(); // Clear initial events

    println!("Run started with default reputation (50)...");

    // Don't adjust reputation - keep it at the default 50
    // Manually end the run
    println!("Ending run with moderate reputation...");
    sim.command(SimCommand::EndRun);
    sim.tick();

    // Poll events to check for ending
    let events = sim.poll_events();
    println!("Events after ending run: {} events", events.len());
    for event in &events {
        match event {
            SimEvent::ShowEnding(view) => {
                println!("  - Unexpected ShowEnding event: id={}", view.id);
            }
            SimEvent::RunCompleted(_) => {
                println!("  - RunCompleted event (expected)");
            }
            _ => {}
        }
    }

    // Check that no ending was triggered
    let ending = find_ending_event(&events);
    assert!(
        ending.is_none(),
        "Expected no ending with moderate reputation, but got {:?}",
        ending
    );

    println!("✓ No ending triggered with moderate reputation");
}

#[test]
fn test_reputation_boundaries() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    println!("=== Testing Reputation Boundaries ===");

    // Start the run
    sim.command(SimCommand::StartRun);
    sim.tick();
    sim.poll_events();

    // Test boundary: reputation slightly above 0 (should not trigger bad ending)
    println!("Testing reputation at 1.0...");
    sim.command(SimCommand::DevAdjustReputation(-49.0)); // 50 -> 1
    sim.tick();
    let events = sim.poll_events();
    for event in &events {
        if let SimEvent::ReputationUpdate(rep) = event {
            println!("  Reputation: {:.2}", rep.reputation);
            assert!(rep.reputation > 0.0, "Reputation should be above 0");
        }
    }

    // End run and check - should NOT trigger ending
    sim.command(SimCommand::EndRun);
    sim.tick();
    let events = sim.poll_events();
    let ending = find_ending_event(&events);
    assert!(
        ending.is_none(),
        "Expected no ending at reputation=1, but got {:?}",
        ending
    );

    println!("✓ No ending at reputation boundary (1.0)");
}
