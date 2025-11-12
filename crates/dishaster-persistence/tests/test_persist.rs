//! Tests for persistence of user progress.

use std::sync::Arc;

use anyhow::Result;
use dishaster_models::*;
use dishaster_persistence::PlayerService;
use dishrupt_persistence::FsStorage;
use tempfile::tempdir;

fn sample_registry() -> GameModelRegistry {
    let mut registry = GameModelRegistry::default();
    let level = LevelConfig {
        id: ModelId::new("level_default"),
        start_day: Day(5),
        run_length: 120.0,
        seed: Seed::new(42),
        canteen: ModelId::new("canteen_default"),
        diner_randomizer: sample_diner_provider(),
        diner_pool: Default::default(),
        window_configurations: vec![],
        table_placements: vec![],
        tray_dispenser_placements: vec![],
        chopstick_dispenser_placements: vec![],
        collector_placements: vec![],
    };
    registry.levels.intern(level.id.clone(), level);
    registry
}

fn sample_diner_provider() -> DinerRandomizerModel {
    DinerRandomizerModel {
        personality: PersonalityRanges {
            frugality: MinMax::new(0.2, 0.8),
            adventurous: MinMax::new(0.2, 0.8),
            confrontational: MinMax::new(0.1, 0.5),
            patience_base: MinMax::new(60.0, 180.0),
            decisiveness: MinMax::new(0.2, 0.8),
            adaptiveness: MinMax::new(0.2, 0.8),
        },
        dining: DiningRanges {
            economic_capacity: MinMax::new(10.0, 20.0),
            max_satiation: MinMax::new(80.0, 130.0),
            eating_speed: MinMax::new(0.5, 1.5),
        },
        appearance: Default::default(),
    }
}

#[test]
fn new_user_receives_first_day_level() -> Result<()> {
    let registry = Arc::new(sample_registry());
    let dir = tempdir()?;
    let service = PlayerService::load_or_create(
        FsStorage::new(dir.path().to_path_buf()).unwrap(),
        &registry,
        None,
    )?;
    let level = service.level_for_current_day()?;
    assert_eq!(level.day, Day(5)); // start_day from LevelConfig
    Ok(())
}
