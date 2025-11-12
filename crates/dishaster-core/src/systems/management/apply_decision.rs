use dishaster_save_models::{CampaignEffect, CampaignTarget, MusicEffect, SloganEffect};

use crate::{
    events::DispatchManagement,
    resources::PermanentEffectsRes,
    systems::{prelude::*, spawn_table},
};

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
        apply_open_window,
        apply_close_window,
        apply_change_window_service,
        apply_play_music,
        apply_advertise_campaign,
        apply_add_motivational_slogan,
    );
}

/// Add tables at random positions
fn apply_add_tables(
    event: On<DispatchManagement<AddTablesModel>>,
    mut commands: Commands,
    registry: Res<GameModelRegistryRes>,
    canteen: Res<Canteen>,
    display_root: Res<DisplayRoot>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;
    for _ in 0..model.num_tables {
        // Spawn at random position
        let table_model_id = registry
            .tables
            .keys()
            .choose(&mut rng)
            .expect("No table models in registry")
            .clone();
        let center_pos = random_table_position(&canteen.model, &mut rng);

        // TODO: check for collisions with existing tables
        let placement = Placement {
            model: table_model_id,
            center_pos,
        };
        spawn_table(&placement, &mut commands, &registry, &display_root);
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
    mut table_query: Query<(Entity, &mut DiningTable, &mut Transform)>,
    canteen: Res<Canteen>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;
    for (_, mut table, mut transform) in table_query
        .iter_mut()
        .choose_multiple(&mut rng, model.num_tables)
    {
        let new_pos = random_table_position(&canteen.model, &mut rng);
        table.center_pos = new_pos;
        transform.position = new_pos.extend(0.);
    }
}

fn random_table_position(canteen: &CanteenModel, rng: &mut Prng) -> Vec2 {
    // TODO: better placement logic
    vec2(
        rng.random_range(1.0..(canteen.width - 1.0)),
        rng.random_range(1.0..(canteen.windows_y - 1.0)),
    )
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

    if let Some(&slot_index) = available_slots.choose(&mut rng) {
        // Now only spawn the components needed for persistence
        commands.spawn(Window {
            service_template: service_handle,
            slot_index,
            location: XSegment::new(0., 0., 0.), // does not matter
            disabled: false,
        });
    }
}

/// Close a random window
fn apply_close_window(
    _event: On<DispatchManagement<CloseWindowModel>>,
    mut commands: Commands,
    mut window_query: Query<Entity>,
    mut rng: ResMut<WorldRng>,
) {
    let Some(window) = window_query.iter_mut().choose(&mut rng) else {
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
