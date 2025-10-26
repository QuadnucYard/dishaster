use dishaster_channel::{events::PresentationEvent, snapshots::DebugSnapshots};
use dishrupt_godot_scene::SceneContext;

use super::{Game, ctrl::*};

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
