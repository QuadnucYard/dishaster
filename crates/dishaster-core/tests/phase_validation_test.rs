//! Tests for phase validation in command handler

mod fixture;

use dishaster_core::sim::Simulation;
use dishaster_interface::{SimCommand, event::SimEvent};
use dishrupt_simulation::ISimulation;
use fixture::{create_test_level, create_test_registry};

fn poll_and_find_phase_error(
    sim: &mut Simulation,
    expected_command: &str,
    expected_phase: &str,
) -> bool {
    let events = sim.poll_events();
    events.iter().any(|e| {
        matches!(
            e,
            SimEvent::PhaseValidationError(e) if e.command_name == expected_command && e.current_phase == expected_phase
        )
    })
}

fn poll_and_ensure_no_phase_error(sim: &mut Simulation, command_name: &str) -> bool {
    let events = sim.poll_events();
    !events.iter().any(|e| {
        matches!(
            e,
            SimEvent::PhaseValidationError (e) if e.command_name == command_name
        )
    })
}

#[test]
fn test_update_pricing_rejected_in_running_phase() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Start run to transition to Running phase
    sim.command(SimCommand::StartRun);
    sim.tick();

    // Try to update pricing in Running phase - should be rejected
    sim.command(SimCommand::UpdateDishPricing {
        dish_entity: dishrupt_core::EntityId::new(1).unwrap(),
        pricing: dishaster_views::PricingMethod::PerPortion(10.0),
    });

    // Check that error event was emitted
    assert!(
        poll_and_find_phase_error(&mut sim, "UpdateDishPricing", "Running"),
        "Expected PhaseValidationError for UpdateDishPricing in Running phase"
    );
}

#[test]
fn test_start_run_rejected_in_running_phase() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Start run to transition to Running phase
    sim.command(SimCommand::StartRun);
    sim.tick();

    // Try to start run again - should be rejected
    sim.command(SimCommand::StartRun);

    assert!(
        poll_and_find_phase_error(&mut sim, "StartRun", "Running"),
        "Expected PhaseValidationError for StartRun in Running phase"
    );
}

#[test]
fn test_end_run_rejected_in_preparation_phase() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Try to end run in Preparation phase - should be rejected
    sim.command(SimCommand::EndRun);

    assert!(
        poll_and_find_phase_error(&mut sim, "EndRun", "Preparation"),
        "Expected PhaseValidationError for EndRun in Preparation phase"
    );
}

#[test]
fn test_apply_management_decision_rejected_in_preparation_phase() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Try to apply management decision in Preparation phase - should be rejected
    sim.command(SimCommand::ApplyManagementDecision(0));

    assert!(
        poll_and_find_phase_error(&mut sim, "ApplyManagementDecision", "Preparation"),
        "Expected PhaseValidationError for ApplyManagementDecision in Preparation phase"
    );
}

#[test]
fn test_apply_management_decision_rejected_in_running_phase() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Start run to transition to Running phase
    sim.command(SimCommand::StartRun);
    sim.tick();

    // Try to apply management decision in Running phase - should be rejected
    sim.command(SimCommand::ApplyManagementDecision(0));

    assert!(
        poll_and_find_phase_error(&mut sim, "ApplyManagementDecision", "Running"),
        "Expected PhaseValidationError for ApplyManagementDecision in Running phase"
    );
}

#[test]
fn test_trial_commands_rejected_in_preparation_phase() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Try trial command in Preparation phase - should be rejected
    sim.command(SimCommand::TrialStart {
        diner: dishrupt_core::EntityId::new(1).unwrap(),
        topic: None,
    });

    assert!(
        poll_and_find_phase_error(&mut sim, "TrialStart", "Preparation"),
        "Expected PhaseValidationError for TrialStart in Preparation phase"
    );
}

#[test]
fn test_commands_accepted_in_correct_phases() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // StartRun should work in Preparation phase
    sim.command(SimCommand::StartRun);
    sim.tick();

    assert!(
        poll_and_ensure_no_phase_error(&mut sim, "StartRun"),
        "StartRun should be accepted in Preparation phase"
    );

    // EndRun should work in Running phase
    sim.command(SimCommand::EndRun);
    sim.tick();

    assert!(
        poll_and_ensure_no_phase_error(&mut sim, "EndRun"),
        "EndRun should be accepted in Running phase"
    );

    // Note: We don't test ApplyManagementDecision here because the command
    // requires actual management decisions to be available, which would
    // require a more complex test setup. The phase validation itself is
    // tested in the rejection tests above.
}

#[test]
fn test_dev_commands_always_allowed() {
    let registry = create_test_registry();
    let level = create_test_level();
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Dev commands should work in all phases
    // In Preparation
    sim.command(SimCommand::DevAdjustReputation(5.0));
    assert!(
        poll_and_ensure_no_phase_error(&mut sim, "DevAdjustReputation"),
        "Dev commands should work in Preparation phase"
    );

    // In Running
    sim.command(SimCommand::StartRun);
    sim.tick();
    sim.command(SimCommand::DevAdjustReputation(5.0));
    assert!(
        poll_and_ensure_no_phase_error(&mut sim, "DevAdjustReputation"),
        "Dev commands should work in Running phase"
    );

    // In Settlement
    sim.command(SimCommand::EndRun);
    sim.tick();
    sim.command(SimCommand::DevAdjustReputation(5.0));
    assert!(
        poll_and_ensure_no_phase_error(&mut sim, "DevAdjustReputation"),
        "Dev commands should work in Settlement phase"
    );
}
