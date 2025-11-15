use super::ordering::{handle_create_session_from_tentative_order, handle_queue_re_evaluation};
use crate::systems::{feedback::*, prelude::*};

/// Check if diners in queue have run out of patience and should abandon
pub fn check_queue_patience(
    mut commands: Commands,
    mut diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerPsychState,
        &mut DinerLongTermMemory,
        &DinerPersonality,
        &QueueMember,
        &mut EntityRng,
    )>,
    lane_query: Query<&QueueLaneMembers>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
) {
    let config = DecisionConfig::default();

    for (entity, mut goal, mut psych_state, mut ltm, _personality, queue_member, mut rng) in
        diner_query.iter_mut()
    {
        if !goal.is(DinerGoal::QueueForWindow) {
            continue;
        }

        // Estimate wait time based on queue position
        let queue_length = lane_query
            .get(queue_member.lane)
            .map(|members| members.members.len())
            .unwrap_or(1);

        let estimated_wait = queue_length as f32 * 10.0; // Rough estimate: 10s per person
        let patience_now = psych_state.patience;

        // Check if patience exceeded
        if estimated_wait > patience_now {
            log::info!(
                target: "diner",
                "diner {:?} abandoning queue due to patience (wait={:.1}s, patience={:.1}s)",
                entity,
                estimated_wait,
                patience_now
            );

            // Apply abandonment penalty
            handle_abandon_penalty(
                &mut psych_state,
                &mut ltm,
                estimated_wait,
                patience_now,
                &config,
            );

            // Additional mood and trust penalties
            psych_state.mood = (psych_state.mood - 0.3).max(-1.0);
            psych_state.trust = (psych_state.trust - 0.15).max(0.0);

            // Emit complaint feedback
            feedback_messages.write(FeedbackMessage {
                entity,
                content: choose_feedback(&mut rng, feedbacks::QUEUE_TOO_LONG),
                trigger: Some(FeedbackTopic::Queue),
            });

            // Leave queue and exit canteen
            commands.entity(entity).remove::<QueueMember>();
            goal.update(DinerGoal::Leave);
        }
    }
}

/// Update psychological state after abandoning a queue
fn handle_abandon_penalty(
    psych_state: &mut PsychState,
    ltm: &mut LongTermMemory,
    estimated_wait: f32,
    patience_now: f32,
    config: &DecisionConfig,
) {
    let excess_ratio = (estimated_wait - patience_now) / patience_now.max(1.0);
    let mood_penalty = config.abandon_mood_penalty * excess_ratio.max(0.0);

    psych_state.mood = (psych_state.mood - mood_penalty).max(-1.0);
    ltm.overall_like = (ltm.overall_like - 0.02).max(0.0);
}

pub fn handle_queue_for_window_goal(
    mut commands: Commands,
    mut diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerState,
        &mut DinerTargets,
        &DinerPersonality,
        &mut DinerPsychState,
        &DinerDiningProfile,
        &DinerLongTermMemory,
        &mut DinerShortTermMemory,
        &mut EntityRng,
        Option<&QueueIntent>,
        Option<&QueueMember>,
    )>,
    window_query: Query<&LaneOwner, With<Window>>,
    window_dishes_query: Query<&WindowDishes>,
    dish_query: Query<&Dish>,
    lane_query: Query<(&QueueLane, &QueueLaneMembers)>,
    registry: Res<GameModelRegistryRes>,
    ordering_config: Res<OrderingConfig>,
    time: Res<Time>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
) {
    for (
        entity,
        mut goal,
        mut diner_state,
        mut targets,
        personality,
        mut psych_state,
        dining_profile,
        ltm,
        mut stm,
        mut rng,
        queue_intent,
        queue_member,
    ) in diner_query.iter_mut()
    {
        if !goal.is(DinerGoal::QueueForWindow) {
            continue;
        }

        // Case 1: Not yet queued - choose lane and join
        if queue_intent.is_none() && queue_member.is_none() {
            choose_lane(
                &mut commands,
                entity,
                &mut goal,
                &targets,
                &window_query,
                &lane_query,
            );
            continue;
        }

        // Case 2: At front of queue - verify and create session
        if let Some(queue_member) = queue_member
            && queue_member.ranking == 0
        {
            handle_create_session_from_tentative_order(
                &mut commands,
                entity,
                &mut goal,
                &mut diner_state,
                &mut targets,
                &mut psych_state,
                &mut rng,
                queue_member,
                &window_dishes_query,
                &dish_query,
                &lane_query,
                &time,
                &mut feedback_messages,
            );
            continue;
        }

        // Re-evaluation rate: 0.02/s means ~1.98% chance per second, or ~9.5% chance in 5 seconds
        // This creates occasional reconsideration without being too frequent
        const RE_EVALUATION_RATE: f64 = 0.02;
        const RE_EVALUATION_COOLDOWN: f32 = 5.0;

        // Case 3: Waiting in queue - allow re-evaluation
        if let Some(queue_member) = queue_member
            && queue_member.ranking > 0
            && goal.timer >= RE_EVALUATION_COOLDOWN
            && rng.random_bool_dt(RE_EVALUATION_RATE, time.tick_duration)
        {
            handle_queue_re_evaluation(
                &mut commands,
                entity,
                &mut goal,
                &mut diner_state,
                &mut targets,
                personality,
                &mut psych_state,
                dining_profile,
                ltm,
                &mut stm,
                &mut rng,
                queue_member,
                &window_dishes_query,
                &dish_query,
                &registry,
                &ordering_config,
                &mut feedback_messages,
            );
        }
    }
}

/// Choose shortest lane and join queue
fn choose_lane(
    commands: &mut Commands,
    entity: Entity,
    goal: &mut DinerGoalState,
    targets: &DinerTargets,
    window_query: &Query<&LaneOwner, With<Window>>,
    lane_query: &Query<(&QueueLane, &QueueLaneMembers)>,
) {
    let Some(window_entity) = targets.chosen_window else {
        // No chosen window, go back to deciding
        goal.update(DinerGoal::DecideWindow);
        return;
    };

    // Choose a lane with the shortest queue of that window
    let lane_entity = window_query
        .get(window_entity)
        .expect("window should exist")
        .lanes
        .iter()
        .map(|&lane_entity| (lane_entity, lane_query.get(lane_entity).unwrap().1))
        .min_by_key(|(_, members)| members.members.len())
        .map(|(lane_entity, _)| lane_entity)
        .expect("window should have at least one lane");

    commands
        .entity(entity)
        .insert(QueueIntent::new(lane_entity));
}
