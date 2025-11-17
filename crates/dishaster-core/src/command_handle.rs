use dishaster_interface::{event::*, response::*, *};
use dishaster_models::InspectorVisitModel;
use dishaster_navigation::*;

use crate::{
    components::*, events::*, messages::*, prelude::*, resources::*, sim::Simulation, views::*,
};

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
                let Some(mut dish) = self.world.get_mut::<Dish>(dish_entity.to_entity()) else {
                    return; // TODO: emit error event
                };

                dish.pricing = pricing.to_model();

                let mut events = self.world.resource_mut::<EventQueue>();
                events.push(SimEvent::DishPriceChanged {
                    entity: dish_entity,
                    new_pricing: pricing,
                });
            }

            SimCommand::RefillDispenser(dispenser_entity) => {
                // Spawn refill staff for the requested dispenser
                self.world.commands().queue(move |world: &mut World| {
                    world.write_message(RefillDispenser(dispenser_entity.to_entity()));
                });
            }

            SimCommand::TrialStart { diner, topic } => {
                self.world.trigger(TrialStart {
                    diner: diner.to_entity(),
                    topic: topic.as_ref().map(ToModel::to_model),
                });
            }
            SimCommand::TrialLaunch => {
                self.world.trigger(TrialLaunch);
            }
            SimCommand::TrialRespond(resp_corpus_index) => {
                self.world.trigger(TrialRespond(resp_corpus_index));
            }
            SimCommand::TrialTimeout => {
                self.world.trigger(TrialTimeout);
            }
            SimCommand::TrialRequestCandidates {
                speech_id,
                keyword_index,
            } => {
                self.world.trigger(TrialRequestCandidates {
                    speech_id,
                    keyword_index,
                });
            }
            SimCommand::TrialProceed => {
                self.world.trigger(TrialProceed);
            }

            SimCommand::ApplyManagementDecision(index) => {
                self.world.trigger(ApplyManagementDecision(index));
            }

            SimCommand::DevAdjustReputation(delta) => {
                // [DEV] Directly adjust reputation for testing
                let mut reputation = self.world.resource_mut::<ReputationStateRes>();
                reputation.reputation = (reputation.reputation + delta).clamp(0.0, 100.0);

                log::info!(
                    "DEV: Adjusted reputation by {:.2}, new value: {:.2}",
                    delta,
                    reputation.reputation
                );

                let rep_view = ReputationView {
                    reputation: reputation.reputation,
                    reputation_delta: delta,
                    fsri: reputation.fsri,
                    food_quality: reputation.food_quality,
                };

                // Emit event to update UI
                let mut events = self.world.resource_mut::<EventQueue>();
                events.push(SimEvent::ReputationUpdate(Box::new(rep_view)));
            }
            SimCommand::DevInspectorVisit(fail) => {
                // [DEV] Trigger inspector visit event
                log::info!("DEV: Triggering inspector visit");
                let model = if fail {
                    InspectorVisitModel {
                        fsri_threshold: 0.0,
                        probability_multiplier: 100.0,
                        reputation_boost: 10.0,
                        trust_boost: 0.15,
                    }
                } else {
                    InspectorVisitModel {
                        fsri_threshold: 15.0,
                        probability_multiplier: 0.05,
                        reputation_boost: 10.0,
                        trust_boost: 0.15,
                    }
                };
                self.world.trigger(InspectorVisit(model));
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
                responses.push(SimResponse::Distances(resp.into()));
            }
        }
    }
}
