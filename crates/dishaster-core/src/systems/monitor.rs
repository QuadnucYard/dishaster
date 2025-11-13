use dishaster_views::ReputationView;

use crate::{events::RunEnded, systems::prelude::*};

/// System to update the current diner count
pub fn check_day_completion(
    mut commands: Commands,
    mut day_status: ResMut<DayStatus>,
    diner_query: Query<&Diner>,
    schedule: Res<DailyDinerSchedule>,
) {
    if day_status.completion_emitted {
        // Day completion already emitted, no further checks needed
        return;
    }

    // Update current diner count
    day_status.live_diner_count = diner_query.iter().count();

    // Check if day is complete: no active diners and no more scheduled arrivals
    let spawning_finished = !schedule.has_pending_spawns();
    day_status.completed = day_status.live_diner_count == 0 && spawning_finished;
    if day_status.completed {
        day_status.completion_emitted = true;
        commands.trigger(RunEnded);
    }
}

/// Emit reputation update event if there are changes
pub fn monitor_reputation_changes(
    reputation: Res<ReputationStateRes>,
    mut events: ResMut<EventQueue>,
) {
    events.push(SimEvent::ReputationUpdate(Box::new(ReputationView {
        reputation: reputation.reputation + reputation.daily_accumulated,
        reputation_delta: reputation.daily_accumulated,
        fsri: reputation.fsri,
        food_quality: reputation.food_quality,
    })));
}
