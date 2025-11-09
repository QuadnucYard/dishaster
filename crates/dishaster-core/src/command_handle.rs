use dishaster_interface::{event::*, response::*, *};
use dishaster_navigation::*;

use crate::{components::*, events::*, messages::*, prelude::*, resources::*, sim::Simulation};

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
                self.world.trigger(RunStarted);
            }
            SimCommand::EndRun => {
                self.world.trigger(RunEnded);
            }

            SimCommand::UpdateDishPricing {
                dish_entity,
                pricing,
            } => {
                if let Some(mut dish) = self.world.get_mut::<Dish>(dish_entity.to_entity()) {
                    dish.pricing = pricing.to_model();
                }
            }

            SimCommand::RefillDispenser(dispenser_entity) => {
                // Spawn refill staff for the requested dispenser
                self.world.commands().queue(move |world: &mut World| {
                    world.write_message(RefillDispenser(dispenser_entity.to_entity()));
                });
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

            SimCommand::ApplyManagementDecision(index) => {
                self.world.trigger(ApplyManagementDecision(index));
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
