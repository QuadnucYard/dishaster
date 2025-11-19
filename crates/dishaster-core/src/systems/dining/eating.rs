use super::deciding::{SatisfactionWeights, compute_satisfaction, update_after_eating};
use crate::systems::{feedback::*, prelude::*};

pub fn handle_eat_goal(
    mut diner_query: Query<(
        Entity,
        &mut DinerGoalState,
        &mut DinerState,
        &mut DinerTargets,
        &DinerPersonality,
        &DinerDiningProfile,
        &mut DinerPsychState,
        &mut DinerLongTermMemory,
        &mut EntityRng,
    )>,
    mut table_query: Query<(Entity, &mut DiningTable)>,
    registry: Res<GameModelRegistryRes>,
    reputation_config: Res<ReputationConfigRes>,
    perma_effects: Res<PermanentEffectsRes>,
    time: Res<Time>,
    mut daily_stats: ResMut<DailyStats>,
    mut feedback_messages: MessageWriter<FeedbackMessage>,
    mut events: ResMut<EventQueue>,
) {
    let satisfaction_weights = SatisfactionWeights::default();
    let feedback_thresholds = &reputation_config.feedback_thresholds;
    let dt = time.tick_duration as f32;

    for (
        entity,
        mut goal,
        mut state,
        mut targets,
        personality,
        dining_profile,
        mut psych_state,
        mut ltm,
        mut rng,
    ) in diner_query.iter_mut()
    {
        if !goal.is(DinerGoal::Eat) {
            continue;
        }

        // Check for contamination during eating (probabilistic detection over time)
        // Check all dishes for contamination
        for served_dish in &state.served_dishes {
            if served_dish.contamination_level > feedback_thresholds.contamination_threshold {
                // Calculate detection chance based on contamination level
                // Higher contamination = faster detection
                let detection_rate = served_dish.contamination_level * 0.5; // 0.05/s at threshold, 0.5/s at max

                if rng.random_bool_dt(detection_rate as f64, dt as f64) {
                    log::warn!(
                        target: "diner",
                        "diner {:?} detected contamination: level={:.2}",
                        entity,
                        served_dish.contamination_level
                    );

                    // Apply severe penalties
                    psych_state.mood = (psych_state.mood - 0.5).max(-1.0);
                    psych_state.trust = (psych_state.trust - 0.3).max(0.0);

                    // Emit strong complaint
                    feedback_messages.write(FeedbackMessage {
                        entity,
                        content: choose_feedback(&mut rng, feedbacks::CONTAMINATION),
                        trigger: Some(FeedbackTopic::Hygiene),
                    });

                    // Stop eating immediately
                    goal.update(DinerGoal::ReturnDishes);
                    continue;
                }
            }
        }

        // Calculate eating progress - probabilistically consume food over time
        // The eating rate depends on dish type and diner's eating speed
        let mut all_finished = true;

        for served_dish in state.served_dishes.iter_mut() {
            if served_dish.remaining_weight <= 0.0 {
                continue; // Already finished this dish
            }

            all_finished = false;

            // Get dish model to access eating_time_per_kg
            let eating_time_per_kg = registry
                .dishes
                .get_by_id(&served_dish.dish_id)
                .map(|m| m.characteristics.eating_time_per_kg)
                .unwrap_or(200.0); // Default fallback

            // Calculate eating rate: kg/second
            // eating_speed is a multiplier (0.5 = slow, 1.0 = normal, 1.5 = fast)
            // eating_time_per_kg is seconds/kg (200 s/kg typical)
            // Apply music effect multiplier from permanent effects
            let eating_time_multiplier = perma_effects.get_eating_time_multiplier();
            // eating_rate = eating_speed / (eating_time_per_kg * multiplier)
            // Lower multiplier = faster eating (less time per kg)
            let eating_rate =
                dining_profile.eating_speed / (eating_time_per_kg * eating_time_multiplier);

            // Expected consumption this tick
            let expected_consumption = eating_rate * dt;

            // Add randomness: actual consumption varies ±50% around expected
            // This makes finish time indeterminate
            let randomness_factor = rng.random_range(0.5..1.5);
            let actual_consumption = expected_consumption * randomness_factor;

            // Consume food (don't go below 0)
            let consumed = actual_consumption.min(served_dish.remaining_weight);
            served_dish.remaining_weight -= consumed;

            // Track consumption in daily stats
            daily_stats.total_consumption_kg += consumed;

            log::trace!(
                target: "diner",
                "eating progress: diner={entity:?} dish={} remaining={:.3}kg rate={:.4}kg/s",
                served_dish.dish_id,
                served_dish.remaining_weight,
                eating_rate
            );
        }

        // Check if all dishes are finished
        if !all_finished {
            continue;
        }

        log::debug!(
            target: "diner",
            "finished eating: diner={entity:?} total_time={:.1}s",
            goal.timer
        );

        // Finished eating - update memory and psychological state for all dishes
        for served_dish in &state.served_dishes {
            // Get dish model for tags and base price
            let dish_tags = registry
                .dishes
                .get_by_id(&served_dish.dish_id)
                .map(|m| m.characteristics.tags.as_slice())
                .unwrap_or(&[]);

            // Use base price from model if available
            let base_price = registry
                .dishes
                .get_by_id(&served_dish.dish_id)
                .and_then(|m| {
                    if m.characteristics.base_price > 0.0 {
                        Some(m.characteristics.base_price)
                    } else {
                        None
                    }
                })
                .unwrap_or(served_dish.price_paid * 0.9);

            // Record hunger before eating
            let hunger_before = psych_state.hunger;

            // Get slogan satisfaction adjustment based on diner trust
            let slogan_adjustment =
                perma_effects.get_slogan_satisfaction_adjustment(psych_state.trust);

            // Calculate satisfaction for feedback
            let satisfaction = compute_satisfaction(
                dish_tags,
                &served_dish.dish_id,
                served_dish.price_paid,
                base_price,
                served_dish.served_quality,
                served_dish.contamination_level,
                hunger_before,
                psych_state.trust,
                slogan_adjustment,
                &ltm,
                &satisfaction_weights,
            );

            // Update diner state
            update_after_eating(
                dish_tags,
                &served_dish.dish_id,
                served_dish.price_paid,
                base_price,
                served_dish.served_quality,
                served_dish.contamination_level,
                time.current_time as f32,
                psych_state.trust,
                slogan_adjustment,
                &mut psych_state,
                &mut ltm,
                &satisfaction_weights,
            );

            // Check for quality mismatch feedback (dish quality below expectations)
            // Calculate expected quality based on memory and base quality
            let expected_quality = compute_expected_quality(&ltm, served_dish, feedback_thresholds);

            // Compute mood-aware quality tolerance
            let quality_tolerance = compute_quality_tolerance(
                personality.adaptiveness,
                psych_state.mood,
                feedback_thresholds.base_quality_tolerance,
            );

            let quality_gap = expected_quality - served_dish.served_quality;
            if quality_gap > quality_tolerance {
                log::info!(
                    target: "diner",
                    "diner {:?} complaining: quality below expectation (expected={:.2}, actual={:.2}, tolerance={:.2})",
                    entity,
                    expected_quality,
                    served_dish.served_quality,
                    quality_tolerance
                );

                feedback_messages.write(FeedbackMessage {
                    entity,
                    content: choose_feedback(&mut rng, feedbacks::BAD_TASTE),
                    trigger: Some(FeedbackTopic::Quality),
                });
            }

            // Check for price complaint (overpriced relative to base)
            let price_ratio = served_dish.price_paid / base_price.max(0.01);
            if price_ratio > feedback_thresholds.max_price_ratio {
                log::info!(
                    target: "diner",
                    "diner {:?} complaining: overpriced (ratio={:.2})",
                    entity,
                    price_ratio
                );

                feedback_messages.write(FeedbackMessage {
                    entity,
                    content: choose_feedback(&mut rng, feedbacks::BAD_TASTE),
                    trigger: Some(FeedbackTopic::Price),
                });
            }

            // Check for praise feedback (high satisfaction)
            // Only praise if satisfaction is positive and no major issues
            if satisfaction > feedback_thresholds.praise_threshold
                && served_dish.contamination_level < 0.05
            {
                log::info!(
                    target: "diner",
                    "diner {:?} praising: good experience (satisfaction={:.2})",
                    entity,
                    satisfaction
                );

                feedback_messages.write(FeedbackMessage {
                    entity,
                    content: choose_feedback(&mut rng, feedbacks::PRAISE),
                    trigger: Some(FeedbackTopic::Praise),
                });
            }

            // Check for bad taste feedback
            if satisfaction < feedback_thresholds.bad_taste_threshold {
                log::info!(
                    target: "diner",
                    "diner {:?} complaining: bad taste (satisfaction={:.2})",
                    entity,
                    satisfaction
                );

                // Emit feedback (emoji only)
                feedback_messages.write(FeedbackMessage {
                    entity,
                    content: choose_feedback(&mut rng, feedbacks::BAD_TASTE),
                    trigger: Some(FeedbackTopic::Taste),
                });
            }

            // Check for still hungry feedback
            if psych_state.hunger > feedback_thresholds.still_hungry_threshold {
                log::info!(
                    target: "diner",
                    "diner {:?} still hungry after eating (hunger={:.2})",
                    entity,
                    psych_state.hunger
                );

                // Mild complaint or thought
                feedback_messages.write(FeedbackMessage {
                    entity,
                    content: choose_feedback(&mut rng, feedbacks::STILL_HUNGRY),
                    trigger: Some(FeedbackTopic::Hunger),
                });
            }
        }

        // Free the table seat
        let (table_entity, seat_index) = targets.chosen_seat.expect("should have chosen seat");
        let mut table = table_query
            .get_mut(table_entity)
            .expect("table should exist")
            .1;
        table.dirtiness += rng.random_range(0.01..0.2); // increase dirtiness. todo: this should be decided by dish and diner
        table.occupants[seat_index] = None; // Free the seat
        targets.chosen_seat = None;

        log::debug!(
            target: "diner",
            "finished_eating: table={table_entity:?} seat={seat_index} pos={:.2}",
            table.seat_positions[seat_index]
        );

        events.push(SimEvent::DinerItemsChanged {
            entity: entity.to_entity_id(),
            change: DinerItemsChange::FinishEating,
        });

        // Track completed diner in daily stats
        daily_stats.completed_diners += 1;

        goal.update(DinerGoal::ReturnDishes);
    }
}

fn compute_expected_quality(
    ltm: &LongTermMemory,
    served_dish: &ServedDish,
    feedback_thresholds: &FeedbackThresholds,
) -> f32 {
    if let Some(dish_mem) = ltm.dish_experience.get(&served_dish.dish_id) {
        // If diner has memory of this dish, use weighted average of memory and base
        let memory_quality = (dish_mem.avg_rating + 1.0) / 2.0; // Map -1..1 to 0..1
        feedback_thresholds.memory_weight * memory_quality
            + feedback_thresholds.base_quality_weight * served_dish.served_quality
    } else {
        // No memory, use base quality as expectation
        served_dish.served_quality
    }
}

/// Helper function to compute quality tolerance based on personality and mood
///
/// Adjusts the quality mismatch tolerance based on diner's mood and adaptiveness.
/// Better mood and more adaptiveness = higher tolerance (more forgiving).
fn compute_quality_tolerance(adaptiveness: f32, mood: f32, base_tolerance: f32) -> f32 {
    base_tolerance + adaptiveness * 0.1 + (mood + 1.0) * 0.05
}
