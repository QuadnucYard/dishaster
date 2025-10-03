use dishaster_core::snapshots::Feedback;
use dishrupt_core::EntityId;
use dishrupt_godot::display::*;
use godot::{
    classes::{Label, Node2D},
    obj::Gd,
};

#[allow(unused)]
pub struct AgentController {
    entity: EntityId,

    root: GdNode2D,
    pub feedback: FeedbackController,
}

impl AgentController {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        let mut feedback = FeedbackController::new(node.get_node_as("Feedback"));
        feedback.root.set_visible(false);

        Self {
            entity,

            feedback,
            root: node,
        }
    }

    pub fn process(&mut self, delta: f64) {
        self.feedback.process(delta);
    }
}

pub struct FeedbackController {
    root: Gd<Node2D>,
    thought: Gd<Node2D>,
    thought_emoji: Gd<Label>,
    speech: Gd<Node2D>,

    lifetime: Option<f64>,
}

impl FeedbackController {
    pub fn new(node: Gd<Node2D>) -> Self {
        Self {
            thought: node.get_node_as("Thought"),
            thought_emoji: node.get_node_as("Thought/Emoji"),
            speech: node.get_node_as("Speech"),
            root: node,

            lifetime: None,
        }
    }

    pub fn process(&mut self, delta: f64) {
        let Some(t) = self.lifetime else {
            return;
        };
        let t = t - delta;
        if t <= 0.0 {
            self.root.set_visible(false);
            self.lifetime = None;
        }
    }

    pub fn show(&mut self, feedback: &Feedback) {
        /// How long feedback balloons last on screen (placeholder value).
        const BALLOON_LIFETIME: f64 = 1.0;

        match feedback {
            Feedback::Thought(emoji) => {
                self.thought_emoji.set_text(emoji.as_str());
                self.thought.set_visible(true);
                self.speech.set_visible(false);
            }
            Feedback::Speech => {
                self.thought.set_visible(false);
                self.speech.set_visible(true);
            }
        }
        self.root.set_visible(true);
        self.lifetime = Some(BALLOON_LIFETIME);
    }
}
