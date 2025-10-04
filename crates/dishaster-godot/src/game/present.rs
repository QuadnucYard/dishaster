use dishaster_core::snapshots::PresentationEvent;

use super::{Game, agent::AgentController};

impl Game {
    pub(crate) fn process_events(&mut self, events: Vec<PresentationEvent>) {
        for event in events {
            match event {
                PresentationEvent::DayCompleted => {
                    // todo
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
            }
        }
    }

    pub(crate) fn process_display(&mut self, delta: f64) {
        for agent in self.agents.values_mut() {
            agent.process(delta);
        }
    }
}
