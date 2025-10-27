use dishaster_channel::{events::PresentationEvent, snapshots::DebugSnapshots};
use dishaster_godot_ui::TrialGui;
use dishrupt_godot_scene::SceneContext;
use dishrupt_godot_ui::UITree;
use godot::global::godot_print;

use super::{Game, ctrl::*};

const TRIAL_FIXED_SIM_TPS: f64 = 30.0;

impl Game {
    pub(crate) fn process_events(
        &mut self,
        ctx: &mut SceneContext,
        events: Vec<PresentationEvent>,
    ) {
        for event in events {
            match event {
                PresentationEvent::DayCompleted => {
                    self.finish_day(ctx, false);
                }
                PresentationEvent::AgentSpawned(entity) => {
                    let mut controller = AgentController::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for agent"),
                    );
                    controller.set_debug_enabled(self.debug_enabled);
                    self.dc.agents.insert(entity, controller);
                }
                PresentationEvent::AgentDespawned(entity) => {
                    self.dc.agents.remove(&entity);
                }
                PresentationEvent::DishSpawned(vm) => {
                    let entity = vm.entity;
                    let mut controller = DishController::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for dish"),
                    );
                    controller.set_view_model(vm);
                    self.dc.dishes.insert(entity, controller);
                }
                PresentationEvent::Feedback(feedback) => {
                    if let Some(agent) = self.dc.agents.get_mut(&feedback.entity) {
                        agent.feedback.show(&feedback.content);
                    }
                }

                PresentationEvent::QueryDistanceResponse(resp) => {
                    godot::global::godot_print!("Distance query response: {:?}", resp);
                }
                PresentationEvent::QueryDistancesResponse(resp) => {
                    self.dbgviz
                        .distance_overlay
                        .present(&resp, self.stage.display_context());
                }

                PresentationEvent::TrialIntro(intro) => {
                    godot_print!("Received trial intro: {:?}", intro);

                    // Force simulation speed to 3x relative to reality during trial
                    self.suspended_sim_speed = Some(self.sim_runner.tps());
                    self.sim_runner.set_tps(TRIAL_FIXED_SIM_TPS);

                    // Show trial GUI
                    let trial_gui = ctx.gui.get_mut::<TrialGui>();
                    trial_gui.intro(intro);
                    trial_gui.show();
                }
                PresentationEvent::TrialLeftSpeak(speech) => {
                    godot_print!("Received trial speech (left): {:?}", speech);

                    let trial_gui = ctx.gui.get_mut::<TrialGui>();
                    trial_gui.left_speak(speech);
                }
                PresentationEvent::TrialRightSpeak(speech) => {
                    godot_print!("Received trial speech (right): {:?}", speech);

                    let trial_gui = ctx.gui.get_mut::<TrialGui>();
                    trial_gui.right_speak(speech);
                }
                PresentationEvent::TrialEnd => {
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
}
