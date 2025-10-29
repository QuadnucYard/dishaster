use rustc_hash::FxHashMap;

use crate::systems::prelude::*;

pub fn populate_diner_pool(
    mut commands: Commands,
    mut level: ResMut<ResWrapper<LevelConfig>>,
    diner_randomizer: Res<ResWrapper<DinerRandomizerModel>>,
    mut rng: ResMut<WorldRng>,
) {
    // Get or create persistent pool from level config
    let mut pool = if level.persistent_diner_pool.is_empty() {
        log::info!("No persistent pool found in level config, creating new one");
        DinerPool::default()
    } else {
        DinerPool {
            profiles: std::mem::take(&mut level.persistent_diner_pool),
            config: DinerPoolConfig::default(),
        }
    };

    // Initialize pool if empty (first run)
    if pool.profiles.is_empty() {
        log::info!(
            "Initializing diner pool with {} profiles",
            pool.config.initial_pool_size
        );

        for i in 0..pool.config.initial_pool_size {
            let profile = create_fresh_diner(i as u32, &diner_randomizer, &mut rng.derive_prng());

            pool.profiles.push(profile);
        }

        log::info!("Pool initialized with {} profiles", pool.profiles.len());
    }

    commands.insert_resource(pool.into_res());
}

fn create_fresh_diner(
    id: u32,
    randomizer: &DinerRandomizerModel,
    rng: &mut impl Rng,
) -> DinerProfile {
    DinerProfile {
        id,
        personality: generate_random_personality(&randomizer.personality, rng),
        dining_profile: generate_random_dining_profile(&randomizer.dining, rng),
        long_term_memory: generate_random_ltm(rng),
        last_visit_day: 0, // Never visited yet
        total_visits: 0,
    }
}

/// Generate a random personality from provider ranges
fn generate_random_personality(pr: &PersonalityRanges, rng: &mut impl Rng) -> Personality {
    Personality {
        frugality: rng.random_range(pr.frugality.min..pr.frugality.max),
        adventurous: rng.random_range(pr.adventurous.min..pr.adventurous.max),
        confrontational: rng.random_range(pr.confrontational.min..pr.confrontational.max),
        patience_base: rng.random_range(pr.patience_base.min..pr.patience_base.max),
        decisiveness: rng.random_range(pr.decisiveness.min..pr.decisiveness.max),
        adaptiveness: rng.random_range(pr.adaptiveness.min..pr.adaptiveness.max),
    }
}

fn generate_random_dining_profile(dr: &DiningRanges, rng: &mut impl Rng) -> DiningProfile {
    // Assign random preferred arrival time
    let service_start = 0.0;
    let service_end = 3600.0;
    let range_width = rng.random_range(600.0..1800.0); // 10-30 min window
    let range_start = rng.random_range(service_start..(service_end - range_width));
    let preferred_arrival_time = (range_start, range_start + range_width);

    // Sample behavioral parameters from randomizer ranges
    let economic_capacity = rng.random_range(dr.economic_capacity.min..dr.economic_capacity.max);
    // Eating speed: 0.5 = slow eater, 1.0 = normal, 1.5 = fast eater
    let eating_speed = rng.random_range(dr.eating_speed.min..dr.eating_speed.max);

    DiningProfile {
        economic_capacity,
        eating_speed,
        preferred_arrival_time,
    }
}

/// Generate random long-term memory with initial tag preferences
fn generate_random_ltm(rng: &mut impl Rng) -> LongTermMemory {
    const POSSIBLE_TAGS: &[&str] = &[
        "meat",
        "vegetable",
        "spicy",
        "mild",
        "soup",
        "rice",
        "noodle",
        "fried",
        "steamed",
        "cold",
        "hot",
    ];

    let mut like_tags = FxHashMap::default();
    for &tag in POSSIBLE_TAGS.iter().take(rng.random_range(3..6)) {
        let preference = rng.random_range(-0.5..0.8); // Slight positive bias
        like_tags.insert(tag.into(), preference);
    }

    LongTermMemory {
        like_tags,
        dish_experience: Default::default(),
        overall_like: 0.5, // Neutral starting satisfaction
    }
}
