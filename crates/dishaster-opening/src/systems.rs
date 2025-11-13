//! Systems for opening animation

use crate::{prelude::*, protocol::SimEvent, resources::OpeningAssetsRes};

/// Spawn food icons based on timer
pub fn spawn_foods(
    mut commands: Commands,
    mut timers: ResMut<SpawnTimers>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
    config: Res<OpeningConfigRes>,
    assets: Res<OpeningAssetsRes>,
    query: Query<&FoodObject>,
    time: Res<DeltaTime>,
) {
    timers.food -= time.delta;

    if timers.food > 0.0 || query.iter().count() >= config.max_foods {
        return;
    }

    let variant = rng.random_range(0..assets.food_variant_count);

    // Randomly spawn from left or right side
    let from_left = rng.random_bool(0.5);

    // Spawn further outside bounds for longer throw trajectories
    let start_y = rng.random_range(config.world_bound.min.y..config.world_bound.max.y);
    let (start_x, vx) = if from_left {
        // From left: throw to the right with higher arc
        (config.world_bound.min.x - 2.0, rng.random_range(4.0..16.0))
    } else {
        // From right: throw to the left with higher arc
        (
            config.world_bound.max.x + 2.0,
            rng.random_range(-16.0..-4.0),
        )
    };

    let position = Position(vec2(start_x, start_y));

    // Strong upward velocity for projectile motion
    let vy = rng.random_range(3.0..8.0);
    let velocity = Velocity(vec2(vx, vy));

    let rotation = Rotation(rng.random_range(0.0..std::f32::consts::TAU));
    let rotation_speed = RotationSpeed(rng.random_range(-5.0..5.0));
    let scale = Scale(rng.random_range(0.6..3.0));

    // Add some color variety (10% chance of bright color tint)
    let color_tint = if rng.random_bool(0.1) {
        ColorTint::random_bright(&mut rng)
    } else {
        ColorTint::white()
    };

    let entity = commands
        .spawn((
            FoodObject {},
            position,
            velocity,
            rotation,
            rotation_speed,
            scale,
            color_tint,
        ))
        .id();

    // Emit spawn event with visual data
    events.push(SimEvent::FoodSpawned {
        entity: entity.to_entity_id(),
        variant,
        color: (color_tint.r, color_tint.g, color_tint.b),
    });

    timers.food = rng.random_range(0.0..config.food_spawn_interval);
}

/// Spawn face icons based on timer
pub fn spawn_faces(
    mut commands: Commands,
    mut timers: ResMut<SpawnTimers>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
    config: Res<OpeningConfigRes>,
    assets: Res<OpeningAssetsRes>,
    query: Query<&FaceObject>,
    time: Res<DeltaTime>,
) {
    timers.face -= time.delta;

    if timers.face > 0.0 || query.iter().count() >= config.max_faces {
        return;
    }

    let variant = rng.random_range(0..assets.face_variant_count);

    // Start from top (slightly outside world bounds) at random horizontal position
    let start_x = rng.random_range(config.world_bound.min.x + 1.0..config.world_bound.max.x - 1.0);
    let position = Position(vec2(start_x, config.world_bound.max.y + 1.0));

    // More varied horizontal drift
    let vx = rng.random_range(-1.5..1.5);
    let velocity = Velocity(vec2(vx, 0.0));

    let rotation = Rotation(rng.random_range(-0.5..0.5));
    let rotation_speed = RotationSpeed(rng.random_range(-1.0..1.0));
    let scale = Scale(rng.random_range(0.6..1.6));

    let entity = commands
        .spawn((
            FaceObject {},
            position,
            velocity,
            rotation,
            rotation_speed,
            scale,
        ))
        .id();

    // Emit spawn event with visual data
    events.push(SimEvent::FaceSpawned {
        entity: entity.to_entity_id(),
        variant,
    });

    timers.face = rng.random_range(0.0..config.face_spawn_interval);
}

/// Spawn review text based on timer
pub fn spawn_texts(
    mut commands: Commands,
    mut timers: ResMut<SpawnTimers>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
    config: Res<OpeningConfigRes>,
    assets: Res<OpeningAssetsRes>,
    query: Query<&TextObject>,
    time: Res<DeltaTime>,
) {
    timers.text -= time.delta;

    if timers.text > 0.0 || query.iter().count() >= config.max_texts {
        return;
    }

    // Pick a random review text from configured corpus
    let content = assets
        .review_texts
        .choose(&mut rng)
        .expect("no review text available")
        .to_string();

    // Random color for text variety
    let text_color = ColorTint::random_bright(&mut rng);

    // Start from right side (outside world bounds) - texts enter from right across full height
    let start_y = rng.random_range(config.world_bound.min.y..config.world_bound.max.y);
    let position = Position(vec2(config.world_bound.max.x + 2.0, start_y));

    let speed = FallSpeed(rng.random_range(0.5..1.5));
    let wave_phase = WavePhase(rng.random_range(0.0..std::f32::consts::TAU));

    let entity = commands
        .spawn((
            TextObject {},
            position,
            speed,
            Alpha(0.0),
            wave_phase,
            text_color,
        ))
        .id();

    // Emit spawn event with text content and color
    events.push(SimEvent::TextSpawned {
        entity: entity.to_entity_id(),
        content,
    });

    timers.text = rng.random_range(0.0..config.text_spawn_interval);
}

/// Update physics for food and face items
pub fn update_physics(
    query: Query<(
        &mut Position,
        &mut Velocity,
        &mut Rotation,
        Option<&RotationSpeed>,
    )>,
    config: Res<OpeningConfigRes>,
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
    query: Query<(&mut Position, &FallSpeed, &mut Alpha, &mut WavePhase), With<TextObject>>,
    config: Res<OpeningConfigRes>,
    time: Res<DeltaTime>,
) {
    let delta = time.delta;
    let fade_distance = 3.0; // Distance for fade in/out effects

    for (mut pos, speed, mut alpha, mut wave) in query {
        // Move left and slightly down - texts travel horizontally across full screen
        // Faster horizontal movement, slower vertical drift
        pos.0 -= speed.0 * 2.0 * delta * vec2(2.0, 0.3);
        // Wave animation
        wave.0 += 2.0 * delta;

        // Fade in when entering from right
        let dist_from_right = config.world_bound.max.x - pos.0.x;
        // Fade out when approaching left edge
        let dist_from_left = pos.0.x - config.world_bound.min.x;
        // Also consider top/bottom bounds for full-screen coverage
        let dist_from_top = config.world_bound.max.y - pos.0.y;
        let dist_from_bottom = pos.0.y - config.world_bound.min.y;

        let fade_in = (dist_from_right / fade_distance).clamp(0.0, 1.0);
        let fade_out_left = (dist_from_left / fade_distance).clamp(0.0, 1.0);
        let fade_vertical =
            ((dist_from_top / fade_distance).min(dist_from_bottom / fade_distance)).clamp(0.0, 1.0);

        alpha.0 = fade_in.min(fade_out_left).min(fade_vertical);
    }
}

/// Despawn items that are outside world bounds
pub fn despawn_out_of_bounds(
    mut commands: Commands,
    mut events: ResMut<EventQueue>,
    query: Query<(Entity, &Position)>,
    config: Res<OpeningConfigRes>,
) {
    // Use a margin of 2 meters to allow items to fully exit the screen before despawning
    let margin = 2.0;
    let region = config.world_bound.inflate(margin);

    for (entity, pos) in query {
        if !region.contains(pos.0) {
            events.push(SimEvent::ObjectDespawned(entity.to_entity_id()));
            commands.entity(entity).despawn();
        }
    }
}
