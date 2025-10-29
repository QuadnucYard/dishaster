use bevy_ecs::system::RunSystemOnce;
use dishaster_interface::{event::*, response::*, *};
use dishaster_navigation::*;

use crate::{components::*, prelude::*, resources::*, sim::Simulation};

impl Simulation {
    /// Apply a high-level control command from the client runtime.
    ///
    /// Commands alter stateful resources directly so that the next simulation tick
    /// reflects the requested transition without delay.
    pub(crate) fn handle_command(&mut self, command: SimCommand) {
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
                let mut schedule = self.world.resource_mut::<DailyDinerSchedule>();
                schedule.finish_spawning();

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

            SimCommand::TrialStart(_entity_id) => {
                // Reset trial session for new trial
                let mut trial_session = self.world.resource_mut::<TrialSession>();
                trial_session.reset();

                // Currently, emit random appearances for both sides.
                let intro = self.create_trial_intro();

                let mut events = self.world.resource_mut::<EventQueue>();
                events.push(SimEvent::TrialIntro(intro));
            }
            SimCommand::TrialLaunch => {
                let statement = self.create_diner_statement();

                let mut events = self.world.resource_mut::<EventQueue>();
                events.push(SimEvent::TrialLeftSpeak(statement));
            }
            SimCommand::TrialRespond(resp_corpus_index) => {
                // Respond with the selected speech
                let speech = self.trial_respond(resp_corpus_index);

                let mut events = self.world.resource_mut::<EventQueue>();
                events.push(SimEvent::TrialRightSpeak(speech));
            }
            SimCommand::TrialTimeout => {
                // TODO: Simply end the trial on timeout for now.
                let mut events = self.world.resource_mut::<EventQueue>();
                events.push(SimEvent::TrialEnd);
            }
            SimCommand::TrialProceed => {
                let should_continue = {
                    let mut session = self.world.resource_mut::<TrialSession>();
                    session.rng.random_bool(0.5)
                };

                if should_continue {
                    // here the logic is same as `TrialLaunch` for now
                    let statement = self.create_diner_statement();

                    let mut events = self.world.resource_mut::<EventQueue>();
                    events.push(SimEvent::TrialLeftSpeak(statement));
                } else {
                    // End of trial
                    let mut events = self.world.resource_mut::<EventQueue>();
                    events.push(SimEvent::TrialEnd);
                }
            }
        }
    }

    pub(crate) fn handle_query(&mut self, query: SimQuery) {
        match query {
            SimQuery::Distance(pos) => {
                let resp = {
                    let nav_grid = self.world.resource_mut::<ResWrapper<NavigationGrid>>();
                    nav_grid
                        .try_world_to_grid(pos)
                        .map(|cell| nav_grid.get_distance(cell))
                };
                let mut responses = self.world.resource_mut::<ResponseQueue>();
                responses.push(SimResponse::Distance(resp));
            }
            SimQuery::Distances => {
                let resp = {
                    let nav_grid = self.world.resource_mut::<ResWrapper<NavigationGrid>>();
                    let distances = nav_grid.distance_field();
                    DistancesResponse {
                        width: distances.rows(),
                        height: distances.cols(),
                        cell_size: nav_grid.cell_size(),
                        data: distances.flatten().clone(),
                    }
                };
                let mut responses = self.world.resource_mut::<ResponseQueue>();
                responses.push(SimResponse::Distances(resp));
            }
        }
    }
}
