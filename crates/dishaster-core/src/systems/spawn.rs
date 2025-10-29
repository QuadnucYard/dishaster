mod diner_provider;
mod diner_schedule;
mod diner_spawner;
mod layout;
mod queues;
mod staffs;

pub use diner_provider::*;
pub use diner_schedule::*;
pub use diner_spawner::*;
pub use layout::*;
pub use queues::*;
pub use staffs::*;

use super::prelude::*;

/// Initial spawning systems to run at level start
pub fn initial_spawning_systems()
-> bevy_ecs::schedule::ScheduleConfigs<Box<dyn System<In = (), Out = ()> + 'static>> {
    (
        spawn_static_objects,
        spawn_window_queues,
        spawn_serving_staffs,
    )
        .chain()
}
