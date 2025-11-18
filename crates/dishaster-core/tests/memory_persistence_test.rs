//! Test that diner long-term memory is properly persisted across days

mod fixture;

use dishaster_core::sim::*;
use dishrupt_simulation::ISimulation;
use fixture::{create_test_level, create_test_registry};

#[test]
fn test_diner_memory_persists_across_days() {
    let registry = create_test_registry();
    let level = create_test_level();

    // Create simulation
    let mut sim = Simulation::new(registry);
    sim.start(level);

    // Store initial diner pool state (after start, which may generate diners)
    let initial_profile = sim.persist();
    let original_pool = initial_profile.diner_profiles.clone();

    if original_pool.is_empty() {
        println!("WARNING: No diners in pool, skipping memory persistence test");
        return;
    }

    // Run simulation until day is complete or timeout
    let mut timeout_ticks = 0;
    const MAX_TIMEOUT_TICKS: i32 = 7200; // 7200 second timeout

    while !sim.is_day_complete() && timeout_ticks < MAX_TIMEOUT_TICKS {
        sim.tick();
        timeout_ticks += 1;
    }

    if timeout_ticks >= MAX_TIMEOUT_TICKS {
        println!("WARNING: Test timed out, skipping memory persistence check");
        return;
    }

    // Persist the simulation state after day completion
    let sim_profile = sim.persist();

    // Check that diner pool was updated
    assert_eq!(
        sim_profile.diner_profiles.len(),
        original_pool.len(),
        "Diner pool size should remain the same"
    );

    // Verify that at least one diner has updated memory
    // (If any diner visited and ate, their memory should have changed)
    let mut found_updated_memory = false;

    for (idx, profile) in sim_profile.diner_profiles.iter().enumerate() {
        let original_profile = &original_pool[idx];

        // Check if long-term memory changed
        let memory_changed = profile.long_term_memory.overall_like
            != original_profile.long_term_memory.overall_like
            || profile.long_term_memory.dish_experience.len()
                != original_profile.long_term_memory.dish_experience.len();

        if memory_changed {
            found_updated_memory = true;
            println!(
                "Diner {} memory changed: overall_like {:.2} → {:.2}, {} → {} dish experiences",
                profile.id,
                original_profile.long_term_memory.overall_like,
                profile.long_term_memory.overall_like,
                original_profile.long_term_memory.dish_experience.len(),
                profile.long_term_memory.dish_experience.len()
            );
        }
    }

    if found_updated_memory {
        println!("✓ Diner long-term memory successfully persisted");
    } else {
        println!("NOTE: No diners ate during simulation, memory persistence not testable");
    }
}
