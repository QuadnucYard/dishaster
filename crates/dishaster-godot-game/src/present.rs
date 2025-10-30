use dishaster_godot_ui::TrialGui;
use dishaster_interface::{snapshots::DebugSnapshots, *};
use dishrupt_godot_scene::SceneContext;
use dishrupt_godot_ui::UITree;
use godot::global::godot_print;

use super::{Game, ctrl::*};

const TRIAL_FIXED_SIM_TPS: f64 = 30.0;

impl Game {
    pub(crate) fn process_events(&mut self, ctx: &mut SceneContext, events: Vec<SimEvent>) {
        for event in events {
            match event {
                SimEvent::DayCompleted => {
                    self.finish_day(ctx, false);
                }
                SimEvent::AgentSpawned { entity, appearance } => {
                    let mut controller = AgentController::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for agent"),
                    );
                    controller.set_debug_enabled(self.debug_enabled);
                    if let Some(appearance) = &appearance {
                        controller.set_appearance(appearance);
                    }
                    self.dc.agents.insert(entity, controller);
                }
                SimEvent::AgentDespawned(entity) => {
                    self.dc.agents.remove(&entity);
                }
                SimEvent::DishSpawned(vm) => {
                    let entity = vm.entity;
                    let mut controller = DishController::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for dish"),
                    );
                    controller.set_view(vm);
                    self.dc.dishes.insert(entity, controller);
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

                    // Show trial GUI
                    let trial_gui = ctx.gui.get_mut::<TrialGui>();
                    trial_gui.intro(intro);
                    trial_gui.show();
                }
                SimEvent::TrialLeftSpeak(statement) => {
                    godot_print!("Received trial speech (left): {:?}", statement);

                    let trial_gui = ctx.gui.get_mut::<TrialGui>();
                    trial_gui.left_speak(statement);
                }
                SimEvent::TrialRightSpeak(speech) => {
                    godot_print!("Received trial speech (right): {:?}", speech);

                    let trial_gui = ctx.gui.get_mut::<TrialGui>();
                    trial_gui.right_speak(speech);
                }
                SimEvent::TrialEnd => {
                    godot_print!("Received trial end");

                    let trial_gui = ctx.gui.get_mut::<TrialGui>();
                    trial_gui.hide();

                    // Restore simulation speed
                    if let Some(speed) = self.suspended_sim_speed.take() {
                        self.sim_runner.set_tps(speed);
                    }
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
                if let Some(agent) = self.dc.agents.get_mut(&diner_debug.core_id) {
                    agent.update_debug(&diner_debug.goal_str);
                }
            }
        }
    }

    pub(crate) fn process_query_responses(
        &mut self,
        _ctx: &mut SceneContext,
        responses: Vec<SimResponse>,
    ) {
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
