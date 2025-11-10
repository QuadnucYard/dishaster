use dishaster_interface::SimEvent;
use dishaster_models::LevelSetupState;

use crate::{components::Diner, events::*, prelude::*, resources::*, systems};

pub fn register_lifecycle_systems(world: &mut World) {
    world.add_observer(on_run_started);
    world.add_observer(on_run_ended);
    world.add_observer(on_advance_day);

    world.add_observer(systems::roll_management_decisions);
    world.add_observer(systems::apply_management_decision);
}

fn on_run_started(_event: On<RunStarted>, mut day_status: ResMut<DayStatus>) {
    day_status.started = true;
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
    mut level: ResMut<ResWrapper<LevelSetupState>>,
    mut events: ResMut<EventQueue>,
) {
    // Update level state for next day. This will be used when persisting progress.
    level.day += 1;
    level.seed = advance_seed(level.seed);
    events.push(SimEvent::Persist);

    // Emit day completed event to advance to next day.
    events.push(SimEvent::DayCompleted);
}

fn advance_seed(seed: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
