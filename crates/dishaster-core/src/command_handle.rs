use bevy_ecs::system::RunSystemOnce;
use dishaster_channel::{commands::SimCommand, events::*};
use dishaster_navigation::*;

use crate::{components::*, prelude::*, resources::*, sim::Simulation};

impl Simulation {
    /// Apply a high-level control command from the client runtime.
    ///
    /// Commands alter stateful resources directly so that the next simulation tick
    /// reflects the requested transition without delay.
    pub(crate) fn handle_command_impl(&mut self, command: SimCommand) {
        match command {
            SimCommand::SetDebugFlags(debug_flags) => {
                self.debug_flags = debug_flags;
            }

            SimCommand::StartRun => {
                let mut day_status = self.world.resource_mut::<DayStatus>();
                day_status.started = true;
            }
            SimCommand::EndRun => {
                // stop spawning
                let mut spawner = self.world.resource_mut::<DinerSpawner>();
                spawner.spawning_finished = true;

                // clear diners
                let _ = self.world.run_system_once(
                    |mut commands: Commands, query: Query<Entity, With<Diner>>| {
                        for entity in query.iter() {
                            commands.entity(entity).despawn();
                        }
                    },
                );
            }

            SimCommand::UpdateDishPricing {
                dish_entity,
                pricing,
            } => {
                if let Some(mut dish) = self.world.get_mut::<Dish>(dish_entity.into()) {
                    dish.pricing = pricing;
                }
            }

            SimCommand::QueryDistance(pos) => {
                let resp = {
                    let nav_grid = self.world.resource_mut::<ResWrapper<NavigationGrid>>();
                    nav_grid
                        .try_world_to_grid(pos)
                        .map(|cell| nav_grid.get_distance(cell))
                };
                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::QueryDistanceResponse(resp));
            }
            SimCommand::QueryDistances => {
                let resp = {
                    let nav_grid = self.world.resource_mut::<ResWrapper<NavigationGrid>>();
                    let distances = nav_grid.distance_field();
                    QueryDistancesResponse {
                        width: distances.rows(),
                        height: distances.cols(),
                        cell_size: nav_grid.cell_size(),
                        data: distances.flatten().clone(),
                    }
                };
                let mut events = self.world.resource_mut::<EventLog>();
                events.emit(PresentationEvent::QueryDistancesResponse(resp));
            }
        }
    }
}
