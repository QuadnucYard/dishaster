use dishaster_interface::SimEvent;

use crate::{components::Diner, events::*, prelude::*, resources::*};

pub fn register_lifecycle_systems(world: &mut World) {
    world.add_observer(on_run_started);
    world.add_observer(on_run_ended);
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
    events.push(SimEvent::DayCompleted);
}
