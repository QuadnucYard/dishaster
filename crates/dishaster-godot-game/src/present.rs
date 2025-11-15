mod agent;
mod diner_items;
mod dish;
mod dispenser;
mod feedback;

use dishaster_interface::{snapshots::*, *};
use dishaster_ui_protocol::{PhaseMusic, UiCommand};
use dishrupt_l10n::tr;
use godot::global::godot_print;

pub use self::{agent::AgentPresenter, dish::DishPresenter, dispenser::DispenserPresenter};
use super::Game;
use crate::{DayPhase, persist::save_sim_profile};

const TRIAL_FIXED_SIM_TPS: f64 = 30.0;

impl Game {
    pub(crate) fn process_events(&mut self, events: Vec<SimEvent>) {
        for event in events {
            match event {
                SimEvent::Persist => {
                    godot_print!("Persisting player progress upon request");

                    let profile = self.sim_runner.persist();
                    save_sim_profile(&self.profile_svc, profile)
                        .expect("failed to persist profile");

                    // Update seen hints in profile
                    let _ = self.profile_svc.update(|profile| {
                        profile.seen_hints = self.hint_tracker.profile_shown_hints().clone();
                        Ok(())
                    });
                }

                SimEvent::RunCompleted => {
                    godot_print!("Process SimEvent: RunCompleted");

                    self.phase = DayPhase::Settlement;

                    // Emit command to transition to settlement music
                    self.ui_commands
                        .push(UiCommand::PlayPhaseMusic(PhaseMusic::Settlement));

                    self.ui_commands.push(UiCommand::FinishRun);
                }
                SimEvent::DayCompleted => {
                    godot_print!("Process SimEvent: DayCompleted");

                    // Advance to next day in level setup
                    self.ui_commands.push(UiCommand::FinishDay);
                }

                SimEvent::ReputationUpdate(view) => {
                    self.ui_commands.push(UiCommand::UpdateReputation(view));
                }

                SimEvent::DispenserSpawned(entity) => {
                    let presenter = DispenserPresenter::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for dispenser"),
                    );
                    self.pres.dispensers.insert(entity, presenter);
                }
                SimEvent::DispenserStockChanged {
                    entity,
                    current_stock,
                    capacity,
                } => {
                    if self.stage.is_entity_valid(entity)
                        && let Some(presenter) = self.pres.dispensers.get_mut(&entity)
                    {
                        presenter.set_stock(current_stock, capacity);
                    }
                }

                SimEvent::DishPriceChanged {
                    entity,
                    new_pricing,
                } => {
                    if self.stage.is_entity_valid(entity)
                        && let Some(presenter) = self.pres.dishes.get_mut(&entity)
                    {
                        presenter.set_price(new_pricing);
                    }
                }

                SimEvent::AgentSpawned { entity, appearance } => {
                    let mut presenter = AgentPresenter::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for agent"),
                    );
                    presenter.set_debug_enabled(self.debug_enabled);
                    if let Some(appearance) = &appearance {
                        presenter.set_appearance(appearance);
                    }
                    self.pres.agents.insert(entity, presenter);
                }
                SimEvent::AgentDespawned(entity) => {
                    self.pres.agents.remove(&entity);
                }
                SimEvent::DishSpawned(vm) => {
                    let entity = vm.entity;
                    let mut presenter = DishPresenter::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for dish"),
                    );
                    presenter.set_view(vm);
                    self.pres.dishes.insert(entity, presenter);
                }
                SimEvent::DinerItemsChanged { entity, change } => {
                    if self.stage.is_entity_valid(entity)
                        && let Some(agent) = self.pres.agents.get_mut(&entity)
                    {
                        agent.handle_item_change(change, &mut self.stage);
                    }
                }
                SimEvent::Feedback(feedback) => {
                    if self.stage.is_entity_valid(feedback.entity)
                        && let Some(agent) = self.pres.agents.get_mut(&feedback.entity)
                        && let Some(feedback_presenter) = &mut agent.feedback
                    {
                        feedback_presenter.show(
                            &feedback.content,
                            feedback.topic,
                            feedback.can_trigger_trial,
                        );
                    }
                }

                SimEvent::TrialIntro(intro) => {
                    godot_print!("Received trial intro: {:?}", intro);

                    // Force simulation speed to 3x relative to reality during trial
                    self.suspended_sim_speed = Some(self.sim_runner.tps());
                    self.sim_runner.set_tps(TRIAL_FIXED_SIM_TPS);

                    // Emit command to enter trial music
                    self.ui_commands.push(UiCommand::EnterTrialMusic);

                    self.ui_commands.push(UiCommand::TrialIntro(intro));
                }
                SimEvent::TrialLeftSpeak(statement) => {
                    godot_print!("Received trial speech (left): {:?}", statement);

                    self.ui_commands.push(UiCommand::TrialLeftSpeak(statement));
                }
                SimEvent::TrialRightSpeak(statement) => {
                    godot_print!("Received trial speech (right): {:?}", statement);

                    self.ui_commands.push(UiCommand::TrialRightSpeak(statement));
                }
                SimEvent::TrialResponseCandidates(options) => {
                    godot_print!(
                        "Received trial response candidates: {} options",
                        options.len()
                    );

                    self.ui_commands
                        .push(UiCommand::TrialResponseCandidates(options));
                }
                SimEvent::TrialImpact(impact) => {
                    godot_print!("Received trial impact: {:?}", impact);

                    self.ui_commands.push(UiCommand::TrialImpact(impact));
                }
                SimEvent::TrialEnd { timeout } => {
                    godot_print!("Received trial end");

                    // Restore simulation speed
                    if let Some(speed) = self.suspended_sim_speed.take() {
                        self.sim_runner.set_tps(speed);
                    }

                    // Emit command to exit trial music
                    self.ui_commands.push(UiCommand::ExitTrialMusic);

                    self.ui_commands.push(UiCommand::TrialEnd { timeout });
                }

                SimEvent::ShowManagementDecisions(view) => {
                    self.ui_commands
                        .push(UiCommand::ShowDecisionSelection(view));
                }
                SimEvent::ShowManagementIncident(view) => {
                    self.ui_commands
                        .push(UiCommand::ShowIncidentNotification(view));
                }

                SimEvent::ShowHint {
                    id: hint_id,
                    condition,
                } => {
                    godot_print!("Received hint: {hint_id}");

                    // Check if we should show this hint based on its emission mode
                    if self.hint_tracker.mark_shown(&hint_id, condition) {
                        let message = tr!(&format!("hint--{hint_id}"));
                        self.ui_commands.push(UiCommand::ShowHint { message });
                    }
                }
            }
        }
    }

    pub(crate) fn process_display(&mut self, delta: f64) {
        for agent in self.pres.agents.values_mut() {
            agent.process(delta, &mut self.stage);
        }
        for dispenser in self.pres.dispensers.values_mut() {
            dispenser.process(delta as f32);
        }
    }

    pub(crate) fn update_other_debug(&mut self, snapshot: &DebugSnapshots) {
        if let Some(diner_debugs) = &snapshot.diners {
            for diner_debug in diner_debugs {
                if let Some(agent) = self.pres.agents.get_mut(&diner_debug.entity) {
                    agent.update_debug(&diner_debug.goal_str);
                }
            }
        }
    }

    pub(crate) fn process_query_responses(&mut self, responses: Vec<SimResponse>) {
        for response in responses {
            match response {
                SimResponse::Distance(resp) => {
                    godot::global::godot_print!("Distance query response: {:?}", resp);
                }
                SimResponse::Distances(resp) => {
                    self.dbgviz
                        .distance_overlay
                        .present(&resp, self.stage.display_context());
                }
            }
        }
    }
}
