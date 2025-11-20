use dishaster_views::InspectorResultView;
use rand_distr::Normal;

use crate::{
    events::{AchieveEnding, DispatchManagement, InspectorVisit},
    systems::{create_fresh_diner, prelude::*},
};

pub fn register_management_incident_systems(world: &mut World) {
    macro_rules! add_observers {
        ($($system: ident),* $(,)?) => {
            $( world.add_observer($system); )*
        };
    }

    add_observers! {
        apply_mislabel_price,
        apply_attraction_change,
        apply_temporary_crowd,
        apply_inspector_visit,
        handle_inspector_visit,
    };
}

fn apply_mislabel_price(
    event: On<DispatchManagement<MislabelPriceModel>>,
    mut dish_query: Query<(Entity, &mut Dish)>,
    mut rng: ResMut<WorldRng>,
    mut events: ResMut<EventQueue>,
) {
    let model = &event.0;
    let mut rng = rng.derive_prng();

    for &rate in &model.overpriced_rates {
        // Choose a dish to mislabel
        let Some((dish_entity, mut dish)) = dish_query.iter_mut().choose(&mut rng) else {
            continue;
        };

        let (original_price, new_price) = match &mut dish.pricing {
            PricingMethod::PerPortion(v) | PricingMethod::ByWeight(v) => {
                let original_price = *v;
                let new_price = original_price * (1.0 + rate);
                *v = new_price;
                (original_price, new_price)
            }
        };

        log::info!(
            "Mislabeling price for dish {:?}: {:.2} -> {:.2}",
            dish.model_id,
            original_price,
            new_price
        );

        events.push(SimEvent::DishPriceChanged {
            entity: dish_entity.to_entity_id(),
            new_pricing: dish.pricing.to_view(),
        });
    }
}

fn apply_attraction_change(
    event: On<DispatchManagement<AttractionChangeModel>>,
    mut perma_effects: ResMut<PermanentEffectsRes>,
) {
    let model = &event.0;

    // Set daily attraction multiplier
    perma_effects.daily_attraction_multiplier = model.attraction_multiplier;

    log::info!(
        "Attraction change incident applied: multiplier = {:.2}",
        model.attraction_multiplier
    );
}

fn apply_temporary_crowd(
    event: On<DispatchManagement<TemporaryCrowdModel>>,
    mut schedule: ResMut<DailyDinerSchedule>,
    registry: Res<GameModelRegistryRes>,
    level: ResMut<ResWrapper<LevelSetupState>>,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;
    let mut rng = rng.derive_prng();

    let level_config = registry
        .levels
        .get_by_id(&level.level_id)
        .expect("LevelConfig not found in registry");

    // Generate temporary diners and add them to a temporary schedule resource
    // These will be merged with the daily schedule when day starts
    let mut temp_diners = Vec::new();

    let distr = Normal::new(model.peak_time, model.time_stddev).unwrap();

    let mut rng = rng.derive_prng();

    for i in 0..model.num_diners {
        // Sample arrival time distributed around peak time
        let arrival_time = distr.sample(&mut rng).max(0.0) * 3600.0; // Convert hours to seconds

        let profile = create_fresh_diner(
            u32::MAX - i as u32, // Use high IDs for temporary diners to avoid conflicts
            &level_config.diner_randomizer,
            &mut rng,
        );

        let hunger = rng.random_range(0.3..1.0);
        let base_budget = profile.dining_profile.economic_capacity * (0.2 + 0.6 * hunger) * 1.25;
        let meal_budget = base_budget * rng.random_range(0.85..1.15);

        temp_diners.push(ScheduledDiner {
            id: u32::MAX - i as u32, // Use high IDs for temporary diners to avoid conflicts
            psych_state: PsychState {
                hunger,
                mood: 0.0,
                patience: profile.personality.patience_base,
                trust: 0.7,
            },
            personality: profile.personality,
            dining_profile: profile.dining_profile,
            long_term_memory: profile.long_term_memory,
            appearance: profile.appearance,
            arrival_time,
            meal_budget,
        });
    }

    // Add temporary diners to the schedule
    schedule.add_many(temp_diners);

    log::info!(
        "Temporary crowd incident: {} diners arriving around {:.2}h",
        model.num_diners,
        model.peak_time
    );
}

fn apply_inspector_visit(
    event: On<DispatchManagement<InspectorVisitModel>>,
    mut commands: Commands,
    mut rng: ResMut<WorldRng>,
) {
    let model = &event.0;

    commands.insert_resource(PendingInspectorVisit {
        model: model.clone(),
        scheduled_time: rng.random_range(0.1..0.3) * 3600.0, // TODO: use actual run length
    });
}

/// Handle inspector visit event (can be triggered from incident or dev command)
fn handle_inspector_visit(
    event: On<InspectorVisit>,
    mut commands: Commands,
    mut reputation: ResMut<ReputationStateRes>,
    mut pool: ResMut<ResWrapper<DinerPool>>,
    mut events: ResMut<EventQueue>,
    mut rng: ResMut<WorldRng>,
) {
    log::info!("Inspector visit incident triggered");
    let InspectorVisitModel {
        fsri_threshold,
        probability_multiplier,
        reputation_boost,
        trust_boost,
    } = event.0;

    let triggers_bad_ending = {
        let excess = reputation.fsri - fsri_threshold;
        let probability = (excess * probability_multiplier).clamp(0.0, 1.0);
        log::info!(
            "Inspector visit check: FSRI = {:.2}, threshold = {:.2}, excess = {:.2}, probability = {:.2}%",
            reputation.fsri,
            fsri_threshold,
            excess,
            probability * 100.0
        );
        rng.random_bool(probability as f64)
    };

    if triggers_bad_ending {
        log::warn!("Inspector visit failed - triggering bad ending!");
        commands.trigger(AchieveEnding(EndingType::Rectification));
        return;
    }

    // Inspection passed - apply reputation and trust boosts (permanent)
    reputation.reputation = (reputation.reputation + reputation_boost).min(100.0);

    // Apply trust boost to existing diners in pool (permanent, affects overall_like)
    for profile in &mut pool.profiles {
        profile.long_term_memory.overall_like =
            (profile.long_term_memory.overall_like + trust_boost).min(1.0);
    }

    log::info!(
        "Inspector visit passed! Reputation +{:.2}, trust +{:.2}",
        reputation_boost,
        trust_boost
    );

    // Show inspector result to player
    events.push(SimEvent::ShowInspectorResult(Box::new(
        InspectorResultView {
            reputation_boost,
            trust_boost,
        },
    )));
}
