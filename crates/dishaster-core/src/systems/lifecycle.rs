use crate::{
    components::Diner,
    events::*,
    interface::{SimEvent, event::HintCondition},
    models::{EndingType, Seed},
    prelude::*,
    resources::*,
    systems::{
        self,
        hint::{HintEmitter, hints},
    },
    views::EndingView,
};

pub fn register_lifecycle_systems(world: &mut World) {
    world.add_observer(on_run_started);
    world.add_observer(on_run_ended);
    world.add_observer(on_advance_day);
    world.add_observer(on_achieve_ending);

    world.add_observer(systems::roll_management_decisions);
    world.add_observer(systems::apply_management_decision);
    world.add_observer(systems::roll_management_incident);

    world.add_observer(systems::apply_trial_impact);

    systems::register_management_decision_systems(world);
    systems::register_management_incident_systems(world);
}

pub fn on_day_started(
    mut perma_effects: ResMut<PermanentEffectsRes>,
    mut events: ResMut<EventQueue>,
) {
    events.emit_hint(hints::ADJUST_PRICE, HintCondition::OnceLocal);

    // Reset daily incident effects at the start of each day
    perma_effects.reset_daily_effects();
}

fn on_run_started(
    _event: On<RunStarted>,
    mut commands: Commands,
    mut time: ResMut<Time>,
    day_status: Res<DayStatus>,
    mut perma_effects: ResMut<PermanentEffectsRes>,
) {
    time.fast_forward_to(day_status.start_time as f64);

    log::info!(
        "Run started for day {} from {}",
        day_status.current_day.0,
        day_status.start_day.0
    );

    // Apply crab effect if present
    if let Some(crab) = perma_effects.crab_trial_probability.take() {
        commands.insert_resource(CrabTurmoil {
            probability: crab,
            trigger_limit: 5,
            triggered_diners: Default::default(),
        });
    }

    if day_status.current_day != day_status.start_day {
        // emit incident for new day
        commands.trigger(RollManagementIncident);
    }
}

fn on_run_ended(
    _event: On<RunEnded>,
    mut commands: Commands,
    diner_query: Query<Entity, With<Diner>>,
    mut schedule: ResMut<DailyDinerSchedule>,
    mut events: ResMut<EventQueue>,
) {
    // Stop spawning
    schedule.finish_spawning();

    // Clear diners
    for entity in diner_query.iter() {
        commands.entity(entity).despawn();
    }

    // Emit day completed event at run end
    events.push(SimEvent::RunCompleted);

    // Trigger management decision roll
    commands.trigger(RollManagementDecisions);
}

fn on_advance_day(
    _event: On<AdvanceDay>,
    mut commands: Commands,
    mut day_status: ResMut<DayStatus>,
    mut perma_effects: ResMut<PermanentEffectsRes>,
    mut reputation: ResMut<ReputationStateRes>,
    reputation_config: Res<ReputationConfigRes>,
    mut events: ResMut<EventQueue>,
) {
    // Apply daily decay to campaign effects
    perma_effects.apply_daily_decay();

    // Apply accumulated reputation changes for the day
    reputation.apply_daily_update(&reputation_config);

    log::info!(
        "Day {} completed. Reputation: {:.1}, FSRI: {:.1}, Quality: {:.1}",
        day_status.current_day.0,
        reputation.reputation,
        reputation.fsri,
        reputation.food_quality
    );

    // Check for reputation-based endings
    if reputation.reputation <= 0.0 {
        log::info!("Reputation dropped to 0 - triggering bad ending");
        commands.trigger(AchieveEnding(EndingType::BadReputation));
    } else if reputation.reputation >= 100.0 {
        log::info!("Reputation reached 100 - potential good ending");
        commands.trigger(AchieveEnding(EndingType::GoodReputation));
    }

    // Update day status for next day. This will be used when persisting progress.
    day_status.current_day.0 += 1;
    day_status.seed = advance_seed(day_status.seed);
    events.push(SimEvent::Persist);

    // Emit day completed event to advance to next day.
    events.push(SimEvent::DayCompleted);
}

fn advance_seed(seed: Seed) -> Seed {
    Seed::new(
        seed.get()
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407),
    )
}

fn on_achieve_ending(event: On<AchieveEnding>, mut events: ResMut<EventQueue>) {
    let ending = event.0;

    events.push(SimEvent::ShowEnding(Box::new(EndingView {
        id: ending.id().into(),
        can_continue: matches!(ending, EndingType::GoodReputation),
    })));
}
