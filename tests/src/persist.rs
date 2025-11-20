//! This is a copy of dishaster-game/src/persist.rs for integration testing.

use anyhow::{Context, Result};
use dishaster_core::models::{
    CanteenLayoutState, CanteenPlacements, GameModelRegistry, LevelConfig, LevelProgress,
    LevelSetupState, ModelId, SimProfile,
};
use dishaster_persistence::ProfileService;

/// Produce a level configuration for the player's current day.
pub fn level_for_current_day(
    svc: &ProfileService,
    registry: &GameModelRegistry,
) -> Result<LevelSetupState> {
    let profile = svc.load().context("failed to load player profile")?;

    let progress = profile.level_progress.unwrap_or_else(|| {
        let default_level =
            get_default_level(registry, None).expect("no default level available in registry");

        LevelProgress {
            level_id: default_level.id.clone(),
            current_day: default_level.start_day,
            reputation: default_level.start_reputation.clone(),
            rng_seed: default_level.seed,
            layout: CanteenLayoutState {
                window_configurations: default_level.window_configurations.clone(),
                placement: CanteenPlacements {
                    tables: default_level.table_placements.clone(),
                    tray_dispensers: default_level.tray_dispenser_placements.clone(),
                    chopstick_dispensers: default_level.chopstick_dispenser_placements.clone(),
                    collectors: default_level.collector_placements.clone(),
                },
            },
            diner_pool: Default::default(),
            permanent_effects: Default::default(),
            daily_history: Vec::new(),
        }
    });

    let level = LevelSetupState {
        level_id: progress.level_id,
        canteen: progress.layout,
        day: progress.current_day,
        seed: progress.rng_seed,
        reputation: progress.reputation,
        diner_pool: progress.diner_pool.profiles,
        permanent_effects: progress.permanent_effects,
    };
    Ok(level)
}

fn get_default_level(
    registry: &GameModelRegistry,
    default_level_id: Option<ModelId>,
) -> Result<&LevelConfig> {
    match default_level_id {
        Some(id) => registry
            .levels
            .get_by_id(&id)
            .context("level does not exist"),
        None => registry
            .levels
            .first()
            .context("no level configurations available in registry"),
    }
}

/// Save simulation profile data after completing a day.
pub fn save_sim_profile(svc: &ProfileService, sim_profile: SimProfile) -> Result<()> {
    svc.update(|profile| {
        // Get or create level progress
        let mut level_progress = profile
            .level_progress
            .take()
            .unwrap_or_else(|| LevelProgress {
                level_id: sim_profile.level_id.clone(),
                current_day: sim_profile.current_day,
                reputation: sim_profile.reputation.clone(),
                rng_seed: sim_profile.rng_seed,
                layout: Default::default(),
                diner_pool: Default::default(),
                permanent_effects: Default::default(),
                daily_history: Vec::new(),
            });

        // Update level progress
        level_progress.level_id = sim_profile.level_id;
        level_progress.current_day = sim_profile.current_day;
        level_progress.reputation = sim_profile.reputation;
        level_progress.rng_seed = sim_profile.rng_seed;
        level_progress.layout.window_configurations = sim_profile.window_configurations;
        level_progress.layout.placement = sim_profile.placement;
        level_progress.diner_pool.profiles = sim_profile.diner_profiles;
        level_progress.permanent_effects = sim_profile.permanent_effects;

        // Update daily history and aggregate stats
        let day_stats = sim_profile.day_stats;
        profile.aggregates.update(&day_stats);
        level_progress.daily_history.push(day_stats);

        profile.level_progress = Some(level_progress);

        Ok(())
    })
}
