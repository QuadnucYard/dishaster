use dishaster_navigation::BoxCollider;
use dishaster_save_models::{CampaignEffect, CampaignTarget, MusicEffect, SloganEffect};

use crate::{
    events::DispatchManagement,
    resources::PermanentEffectsRes,
    systems::{prelude::*, spawn_dispenser, spawn_table},
};

// Safe margin around objects to prevent collisions (in meters)
const COLLISION_SAFE_MARGIN: f32 = 0.5;
const MAX_PLACEMENT_ATTEMPTS: usize = 100;

pub fn register_management_decision_systems(world: &mut World) {
    macro_rules! add_observers {
        ($($system: ident),* $(,)?) => {
            $( world.add_observer($system); )*
        };
    }

    add_observers!(
        apply_add_tables,
        apply_remove_tables,
        apply_disarrange_tables,
        apply_add_dispenser,
        apply_open_window,
        apply_close_window,
        apply_change_window_service,
        apply_play_music,
        apply_advertise_campaign,
        apply_add_motivational_slogan,
        apply_supply_crab,
        apply_improve_dish_quality,
        apply_reduce_serving_time,
    );
}

/// Add tables at random positions
fn apply_add_tables(
    event: On<DispatchManagement<AddTablesModel>>,
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    collider_query: Query<&CompWrapper<BoxCollider>>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;

    // Collect existing colliders
    let existing_colliders: Vec<BoxCollider> = collider_query.iter().map(|c| **c).collect();

    for _ in 0..model.num_tables {
        // Spawn at random position
        let table_model_id = registry
            .tables
            .keys()
            .choose(&mut rng)
            .expect("No table models in registry")
            .clone();

        let table_model = registry
            .tables
            .get_by_id(&table_model_id)
            .expect("Table model not found in registry");
        let table_size = table_model.size.as_vec2();

        // Try to find a non-colliding position
        if let Some(center_pos) =
            find_non_colliding_position(&canteen.model, table_size, &existing_colliders, &mut rng)
        {
            let placement = Placement {
                model: table_model_id,
                center_pos,
            };
            spawn_table(&placement, &mut commands, &registry);
        } else {
            log::warn!(
                "Could not find non-colliding position for table after {} attempts",
                MAX_PLACEMENT_ATTEMPTS
            );
        }
    }
}

/// Remove random tables
fn apply_remove_tables(
    event: On<DispatchManagement<RemoveTablesModel>>,
    mut commands: Commands,
    table_query: Query<Entity, With<DiningTable>>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;
    for entity in table_query
        .iter()
        .choose_multiple(&mut rng, model.num_tables)
    {
        commands.entity(entity).despawn();
    }
}

/// Remove reposition tables
fn apply_disarrange_tables(
    event: On<DispatchManagement<DisarrangeTablesModel>>,
    mut table_query: Query<(
        Entity,
        &mut DiningTable,
        &mut Transform,
        &CompWrapper<BoxCollider>,
    )>,
    collider_query: Query<(Entity, &CompWrapper<BoxCollider>)>,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;

    // Collect existing colliders (excluding the ones we're moving)
    let tables_to_move: Vec<Entity> = table_query
        .iter()
        .choose_multiple(&mut rng, model.num_tables)
        .into_iter()
        .map(|(e, _, _, _)| e)
        .collect();

    let existing_colliders: Vec<BoxCollider> = collider_query
        .iter()
        .filter(|(e, _)| !tables_to_move.contains(e))
        .map(|(_, c)| **c)
        .collect();

    for entity in tables_to_move {
        let Ok((_, mut table, mut transform, _)) = table_query.get_mut(entity) else {
            continue;
        };

        let table_model = registry
            .tables
            .get_by_id(&table.model_id)
            .expect("Table model not found in registry");
        let table_size = table_model.size.as_vec2();

        // Try to find a non-colliding position
        if let Some(new_pos) =
            find_non_colliding_position(&canteen.model, table_size, &existing_colliders, &mut rng)
        {
            table.center_pos = new_pos;
            transform.position = new_pos.extend(0.);
        } else {
            log::warn!(
                "Could not find non-colliding position for table after {} attempts",
                MAX_PLACEMENT_ATTEMPTS
            );
        }
    }
}

/// Add a random dispenser (tray or chopstick)
fn apply_add_dispenser(
    event: On<DispatchManagement<AddDispenserModel>>,
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    collider_query: Query<&CompWrapper<BoxCollider>>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
) {
    let model = &event.0;

    // Collect existing colliders
    let existing_colliders: Vec<BoxCollider> = collider_query.iter().map(|c| **c).collect();

    // Get dispenser model based on type
    let dispenser_model_id = match model.dispenser_type {
        DispenserType::Tray => ModelId::new("tray_dispenser"),
        DispenserType::Chopstick => ModelId::new("chopstick_dispenser"),
    };

    let dispenser_model = registry
        .dispensers
        .get_by_id(&dispenser_model_id)
        .expect("Dispenser model not found in registry");
    let dispenser_size = dispenser_model.size.as_vec2();

    // Try to find a non-colliding position
    if let Some(center_pos) = find_non_colliding_position(
        &canteen.model,
        dispenser_size,
        &existing_colliders,
        &mut rng,
    ) {
        let placement = Placement {
            model: dispenser_model_id,
            center_pos,
        };
        spawn_dispenser(
            &placement,
            &mut commands,
            &registry,
            model.dispenser_type,
            &mut events,
        );
    } else {
        log::warn!(
            "Could not find non-colliding position for dispenser after {} attempts",
            MAX_PLACEMENT_ATTEMPTS
        );
    }
}

/// Find a random position that doesn't collide with existing objects
fn find_non_colliding_position(
    canteen: &CanteenModel,
    object_size: Vec2,
    existing_colliders: &[BoxCollider],
    rng: &mut Prng,
) -> Option<Vec2> {
    for _ in 0..MAX_PLACEMENT_ATTEMPTS {
        let candidate_pos = vec2(
            rng.random_range(1.0..(canteen.width - 1.0)),
            rng.random_range(1.0..(canteen.windows_y - 1.0)),
        );

        // Create a test collider with safe margin
        let test_collider = BoxCollider::from_center_size(
            candidate_pos,
            object_size + vec2(COLLISION_SAFE_MARGIN * 2.0, COLLISION_SAFE_MARGIN * 2.0),
        );

        // Check if it collides with any existing colliders using AABB overlap test
        let has_collision = existing_colliders
            .iter()
            .any(|existing| test_collider.aabb_overlap(existing));

        if !has_collision {
            return Some(candidate_pos);
        }
    }

    None
}

/// Open a new window with random service type
fn apply_open_window(
    _event: On<DispatchManagement<OpenWindowModel>>,
    mut commands: Commands,
    window_query: Query<(Entity, &Window)>,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    mut rng: ResMut<WorldRng>,
) {
    let available_slots = (0..canteen.model.windows.len())
        .filter(|&i| !window_query.iter().any(|(_, w)| w.slot_index == i))
        .collect::<Vec<_>>();

    let service_handle = registry
        .window_services
        .handles()
        .choose(&mut rng)
        .expect("No window models in registry");

    let Some(&slot_index) = available_slots.choose(&mut rng) else {
        log::warn!("No available window slots to open a new window");
        return;
    };

    // Now only spawn the components needed for persistence
    commands.spawn(Window {
        service_template: service_handle,
        slot_index,
        location: XSegment::new(0., 0., 0.), // does not matter
        disabled: false,
    });
}

/// Close a random window
fn apply_close_window(
    _event: On<DispatchManagement<CloseWindowModel>>,
    mut commands: Commands,
    mut window_query: Query<Entity, With<Window>>,
    mut rng: ResMut<WorldRng>,
) {
    log::info!("Applying close window decision");
    let Some(window) = window_query.iter_mut().choose(&mut rng) else {
        log::warn!("No windows available to close");
        return;
    };

    commands.entity(window).despawn();
}

/// Change the service type of a random window
fn apply_change_window_service(
    _event: On<DispatchManagement<ChangeWindowServiceModel>>,
    mut window_query: Query<&mut Window>,
    registry: Res<GameModelRegistryRes>,
    mut rng: ResMut<WorldRng>,
) {
    log::info!("Applying change window service decision");
    let Some(mut window) = window_query.iter_mut().choose(&mut rng) else {
        return;
    };

    let service_handle = registry
        .window_services
        .handles()
        .choose(&mut rng)
        .expect("No window models in registry");

    window.service_template = service_handle;
}

/// Apply music effect (replaces previous music)
fn apply_play_music(
    event: On<DispatchManagement<PlayMusicModel>>,
    mut permanent_effects: ResMut<PermanentEffectsRes>,
) {
    let model = &event.0;

    log::info!(
        "Applying music effect: eating_time_multiplier={:.2}, satisfaction_change={:.2}",
        model.eating_time_multiplier,
        model.satisfaction_change
    );

    // Replace previous music effect
    permanent_effects.music = Some(MusicEffect {
        eating_time_multiplier: model.eating_time_multiplier,
        satisfaction_change: model.satisfaction_change,
    });
}

/// Apply advertising campaign effect
fn apply_advertise_campaign(
    event: On<DispatchManagement<AdvertiseCampaignModel>>,
    mut permanent_effects: ResMut<PermanentEffectsRes>,
    window_query: Query<&Window>,
    registry: Res<GameModelRegistryRes>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;

    // Determine campaign target
    let target = match &model.target {
        DecisionCampaignTarget::Canteen => CampaignTarget::Canteen,
        DecisionCampaignTarget::Window => {
            // Randomly select a window
            if let Some(window) = window_query.iter().choose(&mut rng) {
                let service = registry.window_services.get(window.service_template);
                CampaignTarget::Window(service.id.clone())
            } else {
                log::warn!("No windows available for campaign, falling back to canteen-wide");
                CampaignTarget::Canteen
            }
        }
    };

    log::info!(
        "Applying campaign effect: target={:?}, boost={:.2}, days={}, decay={:.2}",
        target,
        model.attraction_boost,
        model.days_remaining,
        model.decay_rate
    );

    // Add campaign effect (can stack)
    permanent_effects.campaigns.push(CampaignEffect {
        target,
        current_boost: model.attraction_boost,
        days_remaining: model.days_remaining,
        decay_rate: model.decay_rate,
    });
}

/// Apply motivational slogan effect
fn apply_add_motivational_slogan(
    event: On<DispatchManagement<AddMotivationalSloganModel>>,
    mut permanent_effects: ResMut<PermanentEffectsRes>,
) {
    let model = &event.0;

    log::info!(
        "Applying slogan effect: threshold={:.2}, boost={:.2}, penalty={:.2}",
        model.trust_threshold,
        model.satisfaction_boost,
        model.satisfaction_penalty
    );

    // Add slogan effect (can stack)
    permanent_effects.slogans.push(SloganEffect {
        trust_threshold: model.trust_threshold,
        satisfaction_boost: model.satisfaction_boost,
        satisfaction_penalty: model.satisfaction_penalty,
    });
}

/// Apply supply crab decision effect
fn apply_supply_crab(
    event: On<DispatchManagement<SupplyCrabModel>>,
    mut permanent_effects: ResMut<PermanentEffectsRes>,
) {
    let model = &event.0;

    log::info!(
        "Applying supply crab effect: trial_probability={:.2}",
        model.trial_probability
    );

    // Set crab trial probability for next day's diners
    permanent_effects.crab_trial_probability = Some(model.trial_probability);
}

/// Apply dish quality improvement effect
fn apply_improve_dish_quality(
    event: On<DispatchManagement<ImproveDishQualityModel>>,
    mut permanent_effects: ResMut<PermanentEffectsRes>,
) {
    let model = &event.0;

    log::info!(
        "Applying dish quality improvement: multiplier={:.2} (previous={:.2})",
        model.quality_multiplier,
        permanent_effects.dish_quality_multiplier
    );

    // Apply multiplicative stacking to dish quality
    permanent_effects.dish_quality_multiplier *= model.quality_multiplier;
}

/// Apply serving time reduction effect
fn apply_reduce_serving_time(
    event: On<DispatchManagement<ReduceServingTimeModel>>,
    mut permanent_effects: ResMut<PermanentEffectsRes>,
) {
    let model = &event.0;

    log::info!(
        "Applying serving time reduction: multiplier={:.2} (previous={:.2})",
        model.serving_time_multiplier,
        permanent_effects.serving_time_multiplier
    );

    // Apply multiplicative stacking to serving time
    permanent_effects.serving_time_multiplier *= model.serving_time_multiplier;
}
