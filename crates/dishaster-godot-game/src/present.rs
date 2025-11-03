mod agent;
mod dish;
mod feedback;

use dishaster_interface::{snapshots::*, *};
use dishaster_ui_protocol::UiCommand;
use godot::global::godot_print;

pub use self::{agent::AgentPresenter, dish::DishPresenter};
use super::Game;

const TRIAL_FIXED_SIM_TPS: f64 = 30.0;

impl Game {
    pub(crate) fn process_events(&mut self, events: Vec<SimEvent>) {
        for event in events {
            match event {
                SimEvent::DayCompleted => {
                    self.finish_day(false);
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
                    self.dc.agents.insert(entity, presenter);
                }
                SimEvent::AgentDespawned(entity) => {
                    self.dc.agents.remove(&entity);
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
                    self.dc.dishes.insert(entity, presenter);
                }
                SimEvent::DinerItemsChanged(e) => {
                    if let Some(agent) = self.dc.agents.get_mut(&e.entity) {
                        agent.update_items(
                            e.is_eating,
                            e.tray_entity,
                            e.chopsticks_entity,
                            &mut self.stage,
                        );
                    }
                }
                SimEvent::Feedback(feedback) => {
                    if let Some(agent) = self.dc.agents.get_mut(&feedback.entity) {
                        agent.feedback.show(&feedback.content);
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
            }
        }
    }

    pub(crate) fn process_display(&mut self, delta: f64) {
        for agent in self.dc.agents.values_mut() {
            agent.process(delta);
        }
    }

    pub(crate) fn update_other_debug(&mut self, snapshot: &DebugSnapshots) {
        if let Some(diner_debugs) = &snapshot.diners {
            for diner_debug in diner_debugs {
                if let Some(agent) = self.dc.agents.get_mut(&diner_debug.entity) {
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
