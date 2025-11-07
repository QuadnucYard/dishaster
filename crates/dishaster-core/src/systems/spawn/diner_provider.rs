use crate::systems::prelude::*;

pub fn populate_diner_pool(
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    mut level: ResMut<ResWrapper<LevelSetupState>>,
    mut rng: ResMut<WorldRng>,
) {
    let level_config = registry
        .levels
        .get_by_id(&level.level_id)
        .expect("LevelConfig not found in registry");

    // Get or create persistent pool from level config
    let mut pool = if level.diner_pool.is_empty() {
        log::info!("No persistent pool found in level config, creating new one");
        DinerPool::default()
    } else {
        DinerPool {
            profiles: std::mem::take(&mut level.diner_pool),
        }
    };

    // Initialize pool if empty (first run)
    if pool.profiles.is_empty() {
        let pool_size = level_config.diner_pool.initial_pool_size;

        log::info!("Initializing diner pool with {} profiles", pool_size);

        for i in 0..pool_size {
            let profile = create_fresh_diner(
                i as u32,
                &level_config.diner_randomizer,
                &mut rng.derive_prng(),
            );

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
        personality: random_personality(&randomizer.personality, rng),
        dining_profile: random_dining_profile(&randomizer.dining, rng),
        long_term_memory: random_ltm(rng),
        appearance: random_appearance(&randomizer.appearance, rng),
        last_visit_day: 0, // Never visited yet
        total_visits: 0,
    }
}

/// Generate a random personality from provider ranges
fn random_personality(pr: &PersonalityRanges, rng: &mut impl Rng) -> Personality {
    Personality {
        frugality: rng.random_range(pr.frugality.min..pr.frugality.max),
        adventurous: rng.random_range(pr.adventurous.min..pr.adventurous.max),
        confrontational: rng.random_range(pr.confrontational.min..pr.confrontational.max),
        patience_base: rng.random_range(pr.patience_base.min..pr.patience_base.max),
        decisiveness: rng.random_range(pr.decisiveness.min..pr.decisiveness.max),
        adaptiveness: rng.random_range(pr.adaptiveness.min..pr.adaptiveness.max),
    }
}

fn random_dining_profile(dr: &DiningRanges, rng: &mut impl Rng) -> DiningProfile {
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
fn random_ltm(rng: &mut impl Rng) -> LongTermMemory {
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

/// Generate a random appearance within the given ranges
fn random_appearance(ranges: &AppearanceRanges, rng: &mut impl Rng) -> Appearance {
    Appearance {
        head: BodyPart {
            variant: SpriteVariant::new(rng.random_range(0..ranges.head_variants)),
            color_transform: random_color_transform(rng),
        },
        upper_garment: BodyPart {
            variant: SpriteVariant::new(rng.random_range(0..ranges.upper_garment_variants)),
            color_transform: random_color_transform(rng),
        },
        lower_garment: BodyPart {
            variant: SpriteVariant::new(rng.random_range(0..ranges.lower_garment_variants)),
            color_transform: random_color_transform(rng),
        },
        hands: BodyPart {
            variant: SpriteVariant::new(rng.random_range(0..ranges.hand_variants)),
            color_transform: random_color_transform(rng),
        },
        shoes: BodyPart {
            variant: SpriteVariant::new(rng.random_range(0..ranges.shoe_variants)),
            color_transform: random_color_transform(rng),
        },
    }
}

/// Generate a randomized color transform within reasonable ranges
fn random_color_transform(rng: &mut impl Rng) -> ColorTransform {
    ColorTransform {
        // Full hue range for variety
        hue_shift: rng.random_range(0.0..360.0),
        // Saturation between 0.7 and 1.3 (avoid too gray or too vivid)
        saturation: rng.random_range(0.7..1.3),
        // Value between 0.8 and 1.2 (avoid too dark or too bright)
        value: rng.random_range(0.8..1.2),
        // Always fully opaque
        alpha: 1.0,
    }
}
