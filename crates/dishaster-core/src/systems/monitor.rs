use dishaster_views::ReputationView;

use crate::{
    events::{InspectorVisit, RunEnded},
    systems::prelude::*,
};

/// System to update the current diner count
pub fn check_day_completion(
    mut commands: Commands,
    mut day_status: ResMut<DayStatus>,
    diner_query: Query<&Diner>,
    schedule: Res<DailyDinerSchedule>,
    trial_session: Res<TrialSession>,
) {
    if day_status.completion_emitted {
        // Day completion already emitted, no further checks needed
        return;
    }

    // Update current diner count
    day_status.live_diner_count = diner_query.iter().count() as u32;

    // Check if day is complete: no active diners and no more scheduled arrivals
    let spawning_finished = !schedule.has_pending_spawns();
    day_status.completed =
        day_status.live_diner_count == 0 && spawning_finished && !trial_session.is_active;
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

pub fn check_inspector_visit(
    mut commands: Commands,
    time: Res<Time>,
    day_status: Res<DayStatus>,
    pending_visit: Option<Res<PendingInspectorVisit>>,
) {
    let Some(pending_visit) = pending_visit else {
        return;
    };

    // Use relative time for arrival time checks
    let current_time = time.world_time as f32 - day_status.start_time;

    if current_time >= pending_visit.scheduled_time {
        // Trigger the inspector visit event
        commands.trigger(InspectorVisit(pending_visit.model.clone()));
        // Remove the pending visit resource
        commands.remove_resource::<PendingInspectorVisit>();
    }
}
