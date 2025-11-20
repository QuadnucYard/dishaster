//! Test fixtures for dishaster-core tests

use std::sync::Arc;

use dishaster_core::models::*;
use dishrupt_core::{asset::SpriteRef, display::DisplayModel};

/// Create a minimal game model registry for testing
pub fn create_test_registry() -> Arc<GameModelRegistry> {
    let mut registry = GameModelRegistry::default();

    // Add a basic canteen model
    let canteen_model = CanteenModel {
        id: ModelId::new("test_canteen"),
        width: 20.0,
        height: 15.0,
        entrances_y: 0.0,
        entrances: vec![XRange::new(8.0, 12.0)],
        windows_y: 12.0,
        windows: vec![XRange::new(5.0, 15.0)],
        display: DisplayModel::default(),
    };
    registry
        .canteens
        .intern(canteen_model.id.clone(), canteen_model);

    let level_config = LevelConfig {
        id: ModelId::new("test_level"),
        canteen: ModelId::new("test_canteen"),
        start_day: Day(0),
        start_reputation: 50.0,
        entry_time: 39600.0, // 11:00:00
        start_time: 41400.0, // 11:30:00
        run_length: 600.0,
        diner_randomizer: DinerRandomizerModel {
            personality: PersonalityRanges {
                frugality: MinMax::new(0.1, 0.5),
                adventurous: MinMax::new(0.2, 0.8),
                confrontational: MinMax::new(0.1, 0.6),
                patience_base: MinMax::new(60.0, 300.0),
                decisiveness: MinMax::new(0.2, 0.8),
                adaptiveness: MinMax::new(0.1, 0.6),
            },
            dining: DiningRanges {
                economic_capacity: MinMax::new(10.0, 20.0), // Student range
                max_satiation: MinMax::new(80.0, 130.0),    // Small to large appetite
                eating_speed: MinMax::new(0.5, 1.5),
            },
            appearance: Default::default(),
        },
        seed: Default::default(),
        diner_pool: Default::default(),
        window_configurations: vec![],
        table_placements: vec![],
        tray_dispenser_placements: vec![],
        chopstick_dispenser_placements: vec![],
        collector_placements: vec![],
    };
    registry
        .levels
        .intern(level_config.id.clone(), level_config);

    // Add a minimal management decision for testing
    // Using PlayMusic because it's the simplest decision that doesn't require window models
    let decision_template = ManagementDecisionTemplate {
        id: ModelId::new("test_decision"),
        weight: 100,
        icon: SpriteRef::new("test_icon"),
        def: ManagementDecisionTemplateDef::PlayMusic(PlayMusicTemplate {
            eating_time_multiplier_range: 0.9..=1.1,
            satisfaction_change_range: -5.0..=5.0,
        }),
    };
    registry
        .mgmt_decisions
        .intern(decision_template.id.clone(), decision_template);

    Arc::new(registry)
}

/// Create a minimal test level configuration
pub fn create_test_level() -> LevelSetupState {
    LevelSetupState {
        level_id: ModelId::new("test_level"),
        day: Default::default(),
        seed: Default::default(),
        canteen: Default::default(),
        diner_pool: Default::default(),
        permanent_effects: Default::default(),
    }
}
