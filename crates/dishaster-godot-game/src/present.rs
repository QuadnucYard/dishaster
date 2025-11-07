mod agent;
mod diner_items;
mod dish;
mod dispenser;
mod feedback;

use dishaster_interface::{snapshots::*, *};
use dishaster_ui_protocol::UiCommand;
use dishrupt_l10n::tr;
use godot::global::godot_print;

pub use self::{agent::AgentPresenter, dish::DishPresenter, dispenser::DispenserPresenter};
use super::Game;
use crate::progress_service;

const TRIAL_FIXED_SIM_TPS: f64 = 30.0;

impl Game {
    pub(crate) fn process_events(&mut self, events: Vec<SimEvent>) {
        for event in events {
            match event {
                SimEvent::DayCompleted => {
                    self.finish_day(false);
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
                    if let Some(presenter) = self.pres.dispensers.get_mut(&entity) {
                        presenter.set_stock(current_stock, capacity);
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
                    if let Some(agent) = self.pres.agents.get_mut(&entity) {
                        agent.handle_item_change(change, &mut self.stage);
                    }
                }
                SimEvent::Feedback(feedback) => {
                    if let Some(agent) = self.pres.agents.get_mut(&feedback.entity)
                        && let Some(feedback_presenter) = &mut agent.feedback
                    {
                        feedback_presenter.show(&feedback.content);
                    }
                }

                SimEvent::TrialIntro(intro) => {
                    godot_print!("Received trial intro: {:?}", intro);

                    // Force simulation speed to 3x relative to reality during trial
                    self.suspended_sim_speed = Some(self.sim_runner.tps());
                    self.sim_runner.set_tps(TRIAL_FIXED_SIM_TPS);

                    self.ui_commands.push(UiCommand::TrialIntro(intro));
                }
                SimEvent::TrialLeftSpeak(statement) => {
                    godot_print!("Received trial speech (left): {:?}", statement);

                    self.ui_commands.push(UiCommand::TrialLeftSpeak(statement));
                }
                SimEvent::TrialRightSpeak(speech) => {
                    godot_print!("Received trial speech (right): {:?}", speech);

                    self.ui_commands.push(UiCommand::TrialRightSpeak(speech));
                }
                SimEvent::TrialEnd => {
                    godot_print!("Received trial end");

                    // Restore simulation speed
                    if let Some(speed) = self.suspended_sim_speed.take() {
                        self.sim_runner.set_tps(speed);
                    }

                    self.ui_commands.push(UiCommand::TrialEnd);
                }

                SimEvent::ShowHint(hint_id) => {
                    godot_print!("Showing hint: {hint_id}");

                    if self.hint_tracker.mark_shown(&hint_id) {
                        let message = tr!(&format!("hint--{hint_id}"));
                        self.ui_commands.push(UiCommand::ShowHint { message });

                        // Save hint immediately
                        let mut svc = progress_service();
                        svc.update_shown_hint(hint_id);
                        let _ = svc.save(); // Ignore errors for hint saving
                    }
                }
            }
        }
    }

    pub(crate) fn process_display(&mut self, delta: f64) {
        for agent in self.pres.agents.values_mut() {
            agent.process(delta);
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
