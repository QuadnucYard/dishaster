use crate::systems::prelude::*;

pub fn set_daily_schedule(
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    level: Res<ResWrapper<LevelSetupState>>,
    mut pool: ResMut<ResWrapper<DinerPool>>,
    mut rng: ResMut<WorldRng>,
) {
    let level_config = registry
        .levels
        .get_by_id(&level.level_id)
        .expect("LevelConfig not found in registry");
    let schedule = generate_daily_schedule(level_config, &level, &mut pool, &mut rng.derive_prng());
    commands.insert_resource(schedule);
}

/// Generate daily schedule from persistent pool in LevelConfig
///
/// This is called during start() to decide which diners visit today.
/// Core logic:
/// - Iterate persistent pool from level config
/// - Decide visits based on satisfaction memory
/// - Assign arrival times from preferred ranges
fn generate_daily_schedule(
    level_config: &LevelConfig,
    level_setup: &LevelSetupState,
    pool: &mut DinerPool,
    rng: &mut impl Rng,
) -> DailyDinerSchedule {
    let mut scheduled = Vec::new();

    // Current "day" approximation (use level.day if available)
    let current_day = level_setup.day;

    // Decide which existing diners visit today
    for profile in &mut pool.profiles {
        apply_memory_decay(profile, &level_config.diner_pool, current_day);

        // Roll dice
        if !roll_day_visit(profile, &level_config.diner_pool, rng) {
            continue;
        }

        // Sample arrival time from preferred range
        let (min_time, max_time) = profile.dining_profile.preferred_arrival_time;
        let arrival_time = rng.random_range(min_time..=max_time);

        // Randomize hunger for this visit (not persistent)
        let hunger = rng.random_range(0.3..1.0);

        scheduled.push(ScheduledDiner {
            id: profile.id,
            personality: profile.personality.clone(),
            dining_profile: profile.dining_profile.clone(),
            psych_state: PsychState {
                hunger,
                mood: 0.0,
                patience: profile.personality.patience_base * 1.3,
                trust: 0.7,
            },
            long_term_memory: profile.long_term_memory.clone(),
            appearance: profile.appearance.clone(),
            arrival_time,
        });

        profile.last_visit_day = current_day;
        profile.total_visits += 1;
    }

    log::info!(
        "Daily schedule generated: {} diners scheduled from pool of {}",
        scheduled.len(),
        pool.profiles.len()
    );

    DailyDinerSchedule::new(scheduled)
}

fn apply_memory_decay(profile: &mut DinerProfile, config: &DinerPoolConfig, current_day: u32) {
    let days_since_visit = current_day.saturating_sub(profile.last_visit_day);
    if days_since_visit == 0 {
        return;
    }

    let decay_factor = config.memory_decay_rate.powi(days_since_visit as i32);
    profile.long_term_memory.overall_like *= decay_factor;
    profile.long_term_memory.overall_like = profile.long_term_memory.overall_like.clamp(-1.0, 1.0);

    // Decay tag preferences
    let tag_decay = config.tag_decay_rate.powi(days_since_visit as i32);
    for like_value in profile.long_term_memory.like_tags.values_mut() {
        *like_value *= tag_decay;
        *like_value = like_value.clamp(-1.0, 1.0);
    }
}

fn roll_day_visit(profile: &DinerProfile, config: &DinerPoolConfig, rng: &mut impl Rng) -> bool {
    // Calculate visit probability based on satisfaction
    let base_prob = if profile.long_term_memory.overall_like >= 0.0 {
        config.high_satisfaction_visit_rate
    } else {
        config.low_satisfaction_visit_rate
    };

    // Additional modifiers (e.g., frequency bonus for regulars)
    let frequency_bonus = (profile.total_visits as f32 * 0.05).min(0.2);
    let visit_prob = (base_prob + frequency_bonus).min(1.0);

    rng.random_bool(visit_prob as f64)
}
