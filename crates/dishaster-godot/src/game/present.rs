use dishaster_core::snapshots::PresentationEvent;
use dishrupt_godot_scene::SceneContext;

use super::{Game, controllers::*};

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
                    self.dc.agents.insert(entity, controller);
                }
                PresentationEvent::AgentDespawned(entity) => {
                    self.dc.agents.remove(&entity);
                }
                PresentationEvent::DishSpawned(entity, price_str) => {
                    let mut controller = DishController::new(
                        entity,
                        self.stage
                            .get_godot_node(entity)
                            .cloned()
                            .expect("missing godot node for dish"),
                    );
                    controller.set_price(&price_str);
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
}
