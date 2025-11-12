use super::{deciding::*, ordering::*};
use crate::systems::{feedback::*, prelude::*};

pub fn handle_decide_window_goal(
    diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerTargets,
        &DinerState,
        &DinerPersonality,
        &DinerDiningProfile,
        &mut DinerPsychState,
        &DinerLongTermMemory,
        &mut DinerShortTermMemory,
        &mut EntityRng,
    )>,
    window_query: Query<(Entity, &Window, &WindowDishes)>,
    dish_query: Query<&Dish>,
    lane_query: Query<(&QueueLane, &QueueLaneMembers)>,
    registry: Res<GameModelRegistryRes>,
    ordering_config: Res<OrderingConfig>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
) {
    for (
        entity,
        mut goal,
        mut targets,
        diner_state,
        personality,
        dining_profile,
        mut psych_state,
        ltm,
        mut stm,
        mut rng,
    ) in diner_query
    {
        if !goal.is(DinerGoal::DecideWindow) {
            continue;
        }

        // Wait for decision time
        if goal.timer < 3.0 / personality.decisiveness.max(0.1) {
            continue;
        }

        update_patience(personality, &mut psych_state);

        // Evaluate all windows and select one
        let Some(window_entity) = evaluate_and_select_window(
            &window_query,
            &dish_query,
            &lane_query,
            personality,
            &psych_state,
            ltm,
            registry.as_ref(),
            &mut rng,
        ) else {
            handle_no_suitable_window(
                entity,
                &mut goal,
                &mut psych_state,
                &mut rng,
                &mut feedback_messages,
            );
            continue;
        };

        log::info!(target: "diner", "decision: choose_window entity={window_entity:?}");

        // Get window dishes for tentative ordering
        let Some(window_dishes) = window_query
            .get(window_entity)
            .ok()
            .map(|(_, _, dishes)| dishes)
        else {
            log::warn!(
                target: "diner",
                "chosen window {:?} has no dishes, skipping",
                window_entity
            );
            goal.update(DinerGoal::Leave);
            continue;
        };

        // Make tentative order
        let tentative_order = decide_order(
            window_dishes,
            &dish_query,
            personality,
            &psych_state,
            dining_profile,
            ltm,
            &mut stm,
            &registry,
            &ordering_config,
            diner_state.meal_budget,
            &mut rng,
        );

        if tentative_order.is_empty() {
            handle_empty_tentative_order(
                entity,
                &mut goal,
                &mut psych_state,
                &mut rng,
                &mut feedback_messages,
            );
            continue;
        }

        log::debug!(
            target: "diner",
            "diner {:?} formed tentative order with {} dishes",
            entity,
            tentative_order.len()
        );

        // Finalize decision
        finalize_window_choice(
            entity,
            &mut goal,
            &mut targets,
            window_entity,
            tentative_order,
            &mut rng,
            &mut feedback_messages,
        );
    }
}

/// Evaluate all available windows and select the best one
fn evaluate_and_select_window(
    window_query: &Query<(Entity, &Window, &WindowDishes)>,
    dish_query: &Query<&Dish>,
    lane_query: &Query<(&QueueLane, &QueueLaneMembers)>,
    personality: &Personality,
    psych_state: &PsychState,
    ltm: &LongTermMemory,
    registry: &GameModelRegistry,
    rng: &mut EntityRng,
) -> Option<Entity> {
    let config = DecisionConfig::default();
    let mut candidates = Vec::new();

    for (window_entity, window, window_dishes) in window_query.iter() {
        if window.disabled {
            continue;
        }

        let queue_length = lane_query
            .iter()
            .find(|(lane, _)| lane.owner == window_entity)
            .map(|(_, members)| members.members.len())
            .unwrap_or(0);

        let avg_service_time = 10.0; // seconds per person

        if let Some(candidate) = evaluate_window(
            window_entity,
            window_dishes,
            dish_query,
            queue_length,
            avg_service_time,
            personality,
            psych_state,
            ltm,
            registry,
            &config,
        ) {
            candidates.push(candidate);
        }
    }

    select_window_from_candidates(&candidates, &config, rng)
}

/// Handle case when no suitable window is found
fn handle_no_suitable_window(
    entity: Entity,
    goal: &mut DinerGoalState,
    psych_state: &mut PsychState,
    rng: &mut EntityRng,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    log::info!(
        target: "diner",
        "diner {:?} leaving: no appealing dishes found",
        entity
    );

    psych_state.mood = (psych_state.mood - 0.2).max(-1.0);
    psych_state.trust = (psych_state.trust - 0.1).max(0.0);

    feedback_messages.write(FeedbackMessage {
        entity,
        content: choose_feedback(rng, feedbacks::NO_APPEALING_DISH),
        trigger: Some(FeedbackTrigger::NoAppealingDish),
    });

    goal.update(DinerGoal::Leave);
}

/// Handle case when tentative order is empty
fn handle_empty_tentative_order(
    entity: Entity,
    goal: &mut DinerGoalState,
    psych_state: &mut PsychState,
    rng: &mut EntityRng,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    log::info!(
        target: "diner",
        "diner {:?} leaving: cannot form valid order at chosen window",
        entity
    );

    psych_state.mood = (psych_state.mood - 0.2).max(-1.0);
    psych_state.trust = (psych_state.trust - 0.1).max(0.0);

    feedback_messages.write(FeedbackMessage {
        entity,
        content: choose_feedback(rng, feedbacks::NO_APPEALING_DISH),
        trigger: Some(FeedbackTrigger::NoAppealingDish),
    });

    goal.update(DinerGoal::Leave);
}

/// Finalize window choice and store tentative order
fn finalize_window_choice(
    entity: Entity,
    goal: &mut DinerGoalState,
    targets: &mut DinerTargets,
    window_entity: Entity,
    tentative_order: Vec<ServiceRequest>,
    rng: &mut EntityRng,
    feedback_messages: &mut MessageWriter<FeedbackMessage>,
) {
    targets.tentative_order = tentative_order;
    targets.chosen_window = Some(window_entity);

    if rng.random_bool(0.5) {
        feedback_messages.write(FeedbackMessage {
            entity,
            content: choose_feedback(rng, feedbacks::DECIDING),
            trigger: None,
        });
    }

    goal.update(DinerGoal::PickTray);
}
