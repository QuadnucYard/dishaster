use dishaster_core::snapshots::PresentationEvent;
use dishrupt_godot_scene::SceneContext;

use super::{Game, agent::AgentController};

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
                    let controller = AgentController::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for agent"),
                    );
                    self.agents.insert(entity, controller);
                }
                PresentationEvent::AgentDespawned(entity) => {
                    self.agents.remove(&entity);
                }
                PresentationEvent::Feedback(feedback) => {
                    if let Some(agent) = self.agents.get_mut(&feedback.entity) {
                        agent.feedback.show(&feedback.content);
                    }
                }

                PresentationEvent::QueryDistanceResponse(resp) => {
                    godot::global::godot_print!("Distance query response: {:?}", resp);
                }
                PresentationEvent::QueryDistancesResponse(resp) => {
                    self.dbgviz
                        .distance_overlay
                        .present(&resp, &self.display_ctx);
                }
            }
        }
    }

    pub(crate) fn process_display(&mut self, delta: f64) {
        for agent in self.agents.values_mut() {
            agent.process(delta);
        }
    }
}
