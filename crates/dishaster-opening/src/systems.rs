//! Systems for opening animation

use crate::{prelude::*, resources::OpeningAssets};

/// Spawn dish icons based on timer
pub fn spawn_dishes(
    mut commands: Commands,
    mut timers: ResMut<SpawnTimers>,
    mut rng: ResMut<WorldRng>,
    config: Res<OpeningConfig>,
    assets: Res<OpeningAssets>,
    query: Query<&DishIcon>,
    time: Res<DeltaTime>,
) {
    timers.dish += time.delta;

    if timers.dish < config.dish_spawn_interval || query.iter().count() >= config.max_dishes {
        return;
    }

    // Use universal dish prefab (presenter will adjust appearance later)
    let proto = assets.dish_prefab.clone();

    // Start from left side (slightly outside world bounds) at random height
    let start_y = rng.random_range(config.world_bound.min.y + 1.0..config.world_bound.max.y - 1.0);
    let position = Position(vec2(config.world_bound.min.x - 1.0, start_y));

    // Random horizontal velocity and upward arc
    let vx = rng.random_range(3.0..6.0);
    let vy = rng.random_range(-1.0..2.0);
    let velocity = Velocity(vec2(vx, vy));

    let rotation = Rotation(rng.random_range(0.0..std::f32::consts::TAU));
    let rotation_speed = RotationSpeed(rng.random_range(-2.0..2.0));
    let scale = Scale(rng.random_range(0.8..1.5));

    commands.spawn((
        DishIcon { proto },
        position,
        velocity,
        rotation,
        rotation_speed,
        scale,
    ));

    timers.dish = 0.0;
}

/// Spawn emoji icons based on timer
pub fn spawn_emojis(
    mut commands: Commands,
    mut timers: ResMut<SpawnTimers>,
    mut rng: ResMut<WorldRng>,
    config: Res<OpeningConfig>,
    assets: Res<OpeningAssets>,
    query: Query<&EmojiIcon>,
    time: Res<DeltaTime>,
) {
    timers.emoji += time.delta;

    if timers.emoji < config.emoji_spawn_interval || query.iter().count() >= config.max_emojis {
        return;
    }

    // Use universal emoji prefab (presenter will adjust appearance later)
    let proto = assets.emoji_prefab.clone();

    // Start from top (slightly outside world bounds) at random horizontal position
    let start_x = rng.random_range(config.world_bound.min.x + 1.0..config.world_bound.max.x - 1.0);
    let position = Position(vec2(start_x, config.world_bound.max.y + 1.0));

    // Small horizontal drift, no initial vertical velocity
    let vx = rng.random_range(-0.5..0.5);
    let velocity = Velocity(vec2(vx, 0.0));

    let rotation = Rotation(rng.random_range(-0.3..0.3));

    commands.spawn((EmojiIcon { proto }, position, velocity, rotation));

    timers.emoji = 0.0;
}

/// Spawn review text based on timer
pub fn spawn_texts(
    mut commands: Commands,
    mut timers: ResMut<SpawnTimers>,
    mut rng: ResMut<WorldRng>,
    config: Res<OpeningConfig>,
    assets: Res<OpeningAssets>,
    query: Query<&ReviewText>,
    time: Res<DeltaTime>,
) {
    timers.text += time.delta;

    if timers.text < config.text_spawn_interval || query.iter().count() >= config.max_texts {
        return;
    }

    // Pick a random review text from configured corpus
    let content = assets.review_texts.choose(&mut rng).unwrap().to_string();

    // Start from right side (slightly outside world bounds) at random height
    let start_y = rng.random_range(config.world_bound.min.y + 2.0..config.world_bound.max.y - 2.0);
    let position = Position(vec2(config.world_bound.max.x + 1.0, start_y));

    let speed = FallSpeed(rng.random_range(0.5..1.5));
    let wave_phase = WavePhase(rng.random_range(0.0..std::f32::consts::TAU));

    commands.spawn((
        ReviewText { content },
        position,
        speed,
        Alpha(0.0),
        wave_phase,
    ));

    timers.text = 0.0;
}

/// Update physics for dish and emoji items
pub fn update_physics(
    query: Query<(
        &mut Position,
        &mut Velocity,
        &mut Rotation,
        Option<&RotationSpeed>,
    )>,
    config: Res<OpeningConfig>,
    time: Res<DeltaTime>,
) {
    let delta = time.delta;

    for (mut pos, mut vel, mut rot, rot_speed) in query {
        // Update position with velocity
        pos.0 += vel.0 * delta;
        // Apply gravity to vertical velocity
        vel.0.y -= config.gravity * delta;
        // Update rotation if has rotation speed
        if let Some(speed) = rot_speed {
            rot.0 += speed.0 * delta;
        }
    }
}

/// Update review text animation
pub fn update_texts(
    query: Query<(&mut Position, &FallSpeed, &mut Alpha, &mut WavePhase), With<ReviewText>>,
    config: Res<OpeningConfig>,
    time: Res<DeltaTime>,
) {
    let delta = time.delta;
    let fade_distance = 2.0; // Distance for fade in/out effects

    for (mut pos, speed, mut alpha, mut wave) in query {
        // Move down and slightly left
        pos.0.y -= speed.0 * delta;
        pos.0.x -= 0.2 * delta;
        // Wave animation
        wave.0 += 2.0 * delta;

        // Fade in when entering from right
        let dist_from_right = config.world_bound.max.x - pos.0.x;
        // Fade out when approaching bottom or left
        let dist_from_bottom = pos.0.y - config.world_bound.min.y;
        let dist_from_left = pos.0.x - config.world_bound.min.x;

        let fade_in = (dist_from_right / fade_distance).clamp(0.0, 1.0);
        let fade_out_bottom = (dist_from_bottom / fade_distance).clamp(0.0, 1.0);
        let fade_out_left = (dist_from_left / fade_distance).clamp(0.0, 1.0);

        alpha.0 = fade_in.min(fade_out_bottom).min(fade_out_left);
    }
}

/// Despawn items that are outside world bounds
pub fn despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Position)>,
    config: Res<OpeningConfig>,
) {
    // Use a margin of 2 meters to allow items to fully exit the screen before despawning
    let margin = 2.0;
    let region = config.world_bound.inflate(margin);

    for (entity, pos) in query {
        if !region.contains(pos.0) {
            commands.entity(entity).despawn();
        }
    }
}
