use crate::{
    components::DinerState,
    debug::format_feedback_stats,
    events::*,
    interface::{SimEvent, event::HintCondition},
    models::{EndingType, Seed},
    prelude::*,
    resources::*,
    systems::{
        self, despawn_diner_items,
        hint::{HintEmitter, hints},
    },
    views::EndingView,
};

pub fn register_lifecycle_systems(world: &mut World) {
    world.add_observer(on_run_started);
    world.add_observer(on_run_ended);
    world.add_observer(on_confirm_settlement);
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
    mut commands: Commands,
    day_status: Res<DayStatus>,
    mut perma_effects: ResMut<PermanentEffectsRes>,
    mut events: ResMut<EventQueue>,
) {
    // Show tutorial dialog on the first day
    if day_status.current_day == day_status.start_day {
        events.push(SimEvent::ShowTutorial);
    }

    events.emit_hint(hints::ADJUST_PRICE, HintCondition::OnceLocal);

    // Reset daily incident effects at the start of each day
    perma_effects.reset_daily_effects();

    // Show active slogans at day start
    if !perma_effects.slogans.is_empty() {
        events.push(SimEvent::ShowSlogan);
    }

    // Apply crab effect if present
    if let Some(crab) = perma_effects.crab_trial_probability.take() {
        commands.insert_resource(CrabTurmoil {
            probability: crab,
            trigger_limit: 5,
            triggered_diners: Default::default(),
        });
        events.push(SimEvent::ShowCrab);
    }
}

fn on_run_started(
    _event: On<RunStarted>,
    mut commands: Commands,
    mut time: ResMut<Time>,
    mut phase: ResMut<RunPhase>,
    day_status: Res<DayStatus>,
) {
    time.fast_forward_to(day_status.start_time as f64);

    // Transition to Running phase
    *phase = RunPhase::Running;

    log::info!(
        "Run started for day {} from {}",
        day_status.current_day.0,
        day_status.start_day.0
    );

    if day_status.current_day != day_status.start_day {
        // emit incident for new day
        commands.trigger(RollManagementIncident);
    }
}

fn on_run_ended(
    _event: On<RunEnded>,
    mut commands: Commands,
    diner_query: Query<(Entity, &mut DinerState)>,
    mut schedule: ResMut<DailyDinerSchedule>,
    mut phase: ResMut<RunPhase>,
    day_status: Res<DayStatus>,
    daily_stats: Res<DailyStats>,
    reputation: Res<ReputationStateRes>,
    mut events: ResMut<EventQueue>,
) {
    // Stop spawning
    schedule.finish_spawning();

    // Clear diners
    for (entity, mut diner_state) in diner_query {
        despawn_diner_items(&mut commands, &mut diner_state);
        commands.entity(entity).despawn();
    }

    // Transition to Settlement phase
    *phase = RunPhase::Settlement;

    // Create settlement view with day statistics and reputation data
    let settlement_view = Box::new(dishaster_views::SettlementView {
        day: day_status.current_day.0,
        total_visits: daily_stats.total_visits,
        completed_diners: daily_stats.completed_diners,
        revenue: daily_stats.total_revenue,
        consumption_kg: daily_stats.total_consumption_kg,
        avg_serving_time: daily_stats.avg_serving_time(),
        avg_dining_time: daily_stats.avg_dining_time(),
        reputation: reputation.reputation,
        reputation_delta: reputation.daily_accumulated,
        fsri: reputation.fsri,
        food_quality: reputation.food_quality,
    });
    log::info!(
        "Day {} ended. Visits: {}, Completed: {}, Revenue: ¥{:.2}, Consumption: {:.2} kg, Avg Serving Time: {:.1}s, Avg Dining Time: {:.1}s, Reputation: {:.1} ({:+.1}), FSRI: {:.1}, Quality: {:.1}",
        settlement_view.day,
        settlement_view.total_visits,
        settlement_view.completed_diners,
        settlement_view.revenue,
        settlement_view.consumption_kg,
        settlement_view.avg_serving_time,
        settlement_view.avg_dining_time,
        settlement_view.reputation,
        settlement_view.reputation_delta,
        settlement_view.fsri,
        settlement_view.food_quality
    );

    // Emit day completed event at run end with settlement data
    events.push(SimEvent::RunCompleted(settlement_view));
}

fn on_confirm_settlement(
    _event: On<ConfirmSettlement>,
    mut commands: Commands,
    reputation: ResMut<ReputationStateRes>,
) {
    // Check for reputation-based endings
    if reputation.reputation <= 0.0 {
        log::info!("Reputation dropped to 0 - triggering bad ending");
        commands.trigger(AchieveEnding(EndingType::BadReputation));
    } else if reputation.reputation >= 100.0 {
        log::info!("Reputation reached 100 - triggering good ending");
        commands.trigger(AchieveEnding(EndingType::GoodReputation));
        // Decisions will be rolled after player confirms continuation
    } else {
        // No ending - proceed to management decisions
        log::info!("No ending triggered - rolling management decisions");
        commands.trigger(RollManagementDecisions);
    }
}

fn on_advance_day(
    _event: On<AdvanceDay>,
    mut perma_effects: ResMut<PermanentEffectsRes>,
    mut reputation: ResMut<ReputationStateRes>,
    reputation_config: Res<ReputationConfigRes>,
    mut day_status: ResMut<DayStatus>,
    mut events: ResMut<EventQueue>,
) {
    // Apply daily decay to campaign effects
    perma_effects.apply_daily_decay();

    // Log feedback statistics before applying updates
    log::info!(
        "{}",
        format_feedback_stats(&reputation, &reputation_config).unwrap()
    );

    // Apply accumulated reputation changes for the day
    reputation.apply_daily_update(&reputation_config);

    log::info!(
        "Day {} reputation updated. Reputation: {:.1}, FSRI: {:.1}, Quality: {:.1}",
        day_status.current_day.0,
        reputation.reputation,
        reputation.fsri,
        reputation.food_quality
    );

    // Update day status for next day. This will be used when persisting progress.
    day_status.current_day.0 += 1;
    day_status.seed = advance_seed(day_status.seed);

    // Persist reputation state immediately after updating
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

    // If the ending allows continuation, roll management decisions after showing the ending
    // This will be triggered after the player sees the ending dialog
}
