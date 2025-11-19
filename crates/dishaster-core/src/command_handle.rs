use dishaster_interface::{event::*, response::*, *};
use dishaster_models::InspectorVisitModel;
use dishaster_navigation::*;

use crate::{
    components::*, debug::format_feedback_stats, events::*, messages::*, prelude::*, resources::*,
    sim::Simulation, views::*,
};

impl Simulation {
    /// Apply a high-level control command from the client runtime.
    ///
    /// Commands alter stateful resources directly so that the next simulation tick
    /// reflects the requested transition without delay.
    pub(crate) fn handle_command(&mut self, command: SimCommand) {
        // Validate command against current phase
        if let Err(error) = self.validate_phase(&command) {
            let mut events = self.world.resource_mut::<EventQueue>();
            events.push(error);
            return;
        }

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
            SimCommand::TrialRespond(resp_id) => {
                self.world.trigger(TrialRespond(resp_id));
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
            SimCommand::DevCrab => {
                // [DEV] Trigger crab turmoil event
                log::info!("DEV: Triggering crab turmoil");
                self.world.insert_resource(CrabTurmoil {
                    probability: 0.001,
                    trigger_limit: 5,
                    triggered_diners: Default::default(),
                });
            }
        }
    }

    pub(crate) fn handle_query(&mut self, query: SimQuery) {
        let resp = self.execute_query(query);
        let mut responses = self.world.resource_mut::<ResponseQueue>();
        responses.push(resp);
    }

    /// Execute a simulation query immediately and return the response.
    pub fn execute_query(&mut self, query: SimQuery) -> SimResponse {
        match query {
            SimQuery::Distance(pos) => {
                let nav_grid = self.world.resource::<ResWrapper<NavigationGrid>>();
                let distance = nav_grid
                    .try_world_to_grid(pos)
                    .map(|cell| nav_grid.get_distance(cell));
                SimResponse::Distance(distance)
            }
            SimQuery::Distances => {
                let nav_grid = self.world.resource_mut::<ResWrapper<NavigationGrid>>();
                let distances = nav_grid.distance_field();
                SimResponse::Distances(Box::new(DistancesResponse {
                    width: distances.rows(),
                    height: distances.cols(),
                    cell_size: nav_grid.cell_size(),
                    data: distances.flatten().clone(),
                }))
            }
            SimQuery::FeedbackStats => {
                let reputation = self.world.resource::<ReputationStateRes>();
                let config = self.world.resource::<ReputationConfigRes>();
                let stats = format_feedback_stats(reputation, config).unwrap();
                SimResponse::FeedbackStats(stats)
            }
        }
    }

    /// Validate command against current phase.
    ///
    /// Returns Ok(()) if the command is allowed in the current phase,
    /// otherwise returns an error event to be emitted.
    fn validate_phase(&self, command: &SimCommand) -> Result<(), SimEvent> {
        use SimCommand::*;

        let phase = *self.world.resource::<RunPhase>();

        // Define which commands are allowed in which phases
        let valid_phases: &[RunPhase] = match command {
            // Debug commands are always allowed
            SetDebugFlags(_) | DevAdjustReputation(_) | DevInspectorVisit(_) | DevCrab => &[
                RunPhase::Preparation,
                RunPhase::Running,
                RunPhase::Settlement,
            ],

            // Preparation phase only
            UpdateDishPricing { .. } => &[RunPhase::Preparation],
            StartRun => &[RunPhase::Preparation],

            // Running phase only
            RefillDispenser(_) => &[RunPhase::Running],
            TrialStart { .. }
            | TrialLaunch
            | TrialRespond(_)
            | TrialTimeout
            | TrialRequestCandidates { .. }
            | TrialProceed => &[RunPhase::Running],
            EndRun => &[RunPhase::Running],

            // Settlement phase only
            ApplyManagementDecision(_) => &[RunPhase::Settlement],
        };

        if valid_phases.contains(&phase) {
            Ok(())
        } else {
            Err(SimEvent::PhaseValidationError(Box::new(
                PhaseValidationError {
                    command_name: command_name(command).into(),
                    current_phase: phase_name(phase).into(),
                    valid_phases: valid_phases.iter().map(|p| phase_name(*p).into()).collect(),
                },
            )))
        }
    }
}

fn command_name(command: &SimCommand) -> &'static str {
    match command {
        SimCommand::SetDebugFlags(_) => "SetDebugFlags",
        SimCommand::StartRun => "StartRun",
        SimCommand::EndRun => "EndRun",
        SimCommand::UpdateDishPricing { .. } => "UpdateDishPricing",
        SimCommand::RefillDispenser(_) => "RefillDispenser",
        SimCommand::TrialStart { .. } => "TrialStart",
        SimCommand::TrialLaunch => "TrialLaunch",
        SimCommand::TrialRespond(_) => "TrialRespond",
        SimCommand::TrialTimeout => "TrialTimeout",
        SimCommand::TrialRequestCandidates { .. } => "TrialRequestCandidates",
        SimCommand::TrialProceed => "TrialProceed",
        SimCommand::ApplyManagementDecision(_) => "ApplyManagementDecision",
        SimCommand::DevAdjustReputation(_) => "DevAdjustReputation",
        SimCommand::DevInspectorVisit(_) => "DevInspectorVisit",
        SimCommand::DevCrab => "DevCrab",
    }
}

fn phase_name(phase: RunPhase) -> &'static str {
    match phase {
        RunPhase::Preparation => "Preparation",
        RunPhase::Running => "Running",
        RunPhase::Settlement => "Settlement",
    }
}
