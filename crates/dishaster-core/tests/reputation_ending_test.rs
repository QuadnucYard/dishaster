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
    // Should have settlement view
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::RunCompleted(_))),
        "Expected RunCompleted event with settlement view"
    );

    // Confirm settlement to trigger ending check
    println!("Confirming settlement...");
    sim.command(SimCommand::ConfirmSettlement);
    sim.tick();

    // Poll events to check for ending
    let events = sim.poll_events();
    println!(
        "Events after confirming settlement: {} events",
        events.len()
    );
    for event in &events {
        if let SimEvent::ShowEnding(view) = event {
            println!(
                "  - ShowEnding event: id={}, can_continue={}",
                view.id, view.can_continue
            );
        }
    }

    // Check that bad reputation ending was triggered
    let ending = find_ending_event(&events);
    assert!(
        ending.is_some(),
        "Expected ending event to be emitted after ConfirmSettlement, but none found"
    );
    assert_eq!(
        ending.unwrap(),
        EndingType::BadReputation,
        "Expected BadReputation ending when reputation is 0"
    );

    // Bad reputation ending should NOT show management decisions (cannot continue)
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_))),
        "Management decisions should not be shown for bad ending (game over)"
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
    // Should have settlement view
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::RunCompleted(_))),
        "Expected RunCompleted event with settlement view"
    );

    // Confirm settlement to trigger ending check
    println!("Confirming settlement...");
    sim.command(SimCommand::ConfirmSettlement);
    sim.tick();

    // Poll events to check for ending
    let events = sim.poll_events();
    println!(
        "Events after confirming settlement: {} events",
        events.len()
    );
    for event in &events {
        match event {
            SimEvent::ShowEnding(view) => {
                println!(
                    "  - ShowEnding event: id={}, can_continue={}",
                    view.id, view.can_continue
                );
            }
            SimEvent::ShowManagementDecisions(_) => {
                println!("  - ShowManagementDecisions event");
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
        "Expected ending event to be emitted after ConfirmSettlement, but none found"
    );
    assert_eq!(
        ending.unwrap(),
        EndingType::GoodReputation,
        "Expected GoodReputation ending when reputation is 100"
    );

    // Good ending shows WITHOUT decisions yet - decisions only appear after player confirms continuation
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_))),
        "Management decisions should NOT be shown yet (waiting for player to confirm continuation)"
    );

    // Now the player confirms continuation from the ending screen
    println!("Continuing from ending...");
    sim.command(SimCommand::ContinueFromEnding);
    sim.tick();

    // Now decisions should be rolled
    let events = sim.poll_events();
    println!(
        "Events after continuing from ending: {} events",
        events.len()
    );
    for event in &events {
        if let SimEvent::ShowManagementDecisions(_) = event {
            println!("  - ShowManagementDecisions event (expected)");
        }
    }

    // Verify that decisions were shown after confirming continuation
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_))),
        "Management decisions should be shown after confirming continuation from good ending"
    );

    // Now apply a decision to advance to next day
    println!("Applying management decision to advance day...");
    sim.command(SimCommand::ApplyManagementDecision(0));
    sim.tick();

    let events = sim.poll_events();
    println!("Events after applying decision: {} events", events.len());
    if events.iter().any(|e| matches!(e, SimEvent::DayCompleted)) {
        println!("  - DayCompleted event");
    }

    // Verify day advanced
    assert!(
        events.iter().any(|e| matches!(e, SimEvent::DayCompleted)),
        "Expected DayCompleted event after applying decision"
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

    // Poll events to check for settlement
    let events = sim.poll_events();
    println!("Events after ending run: {} events", events.len());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::RunCompleted(_))),
        "Expected RunCompleted event with settlement view"
    );

    // Confirm settlement to trigger ending check
    println!("Confirming settlement...");
    sim.command(SimCommand::ConfirmSettlement);
    sim.tick();

    // Poll events after confirming settlement
    let events = sim.poll_events();
    println!(
        "Events after confirming settlement: {} events",
        events.len()
    );
    for event in &events {
        match event {
            SimEvent::ShowEnding(view) => {
                println!("  - Unexpected ShowEnding event: id={}", view.id);
            }
            SimEvent::ShowManagementDecisions(_) => {
                println!("  - ShowManagementDecisions event (expected)");
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

    // Should show management decisions instead
    assert!(
        events
            .iter()
            .any(|e| matches!(e, SimEvent::ShowManagementDecisions(_))),
        "Expected management decisions to be shown when no ending triggered"
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
