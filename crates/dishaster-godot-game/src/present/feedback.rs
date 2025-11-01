use dishaster_ui_protocol::UiCommand;
use dishaster_views::Feedback;
use dishrupt_core::EntityId;
use dishrupt_godot::input::event::MouseButtonEvent;
use godot::{
    classes::{Label, Node2D},
    prelude::*,
};

use crate::input::{Pickable, PickingContext};

pub struct FeedbackPresenter {
    /// The agent entity this feedback belongs to.
    entity: EntityId,

    root: Gd<Node2D>, // actually Area2D
    thought: Gd<Node2D>,
    thought_emoji: Gd<Label>,
    speech: Gd<Node2D>,

    lifetime: Option<f64>,
}

impl FeedbackPresenter {
    pub fn new(entity: EntityId, node: Gd<Node2D>) -> Self {
        Self {
            entity,

            thought: node.get_node_as("Thought"),
            thought_emoji: node.get_node_as("Thought/Emoji"),
            speech: node.get_node_as("Speech"),
            root: node,

            lifetime: None,
        }
    }

    pub fn process(&mut self, delta: f64) {
        let Some(t) = &mut self.lifetime else {
            return;
        };
        *t -= delta;
        if *t <= 0.0 {
            self.root.set_visible(false);
            self.lifetime = None;
        }
    }

    pub fn show(&mut self, feedback: &Feedback) {
        /// How long feedback balloons last on screen (placeholder value).
        const BALLOON_LIFETIME: f64 = 3.0;

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

    pub fn hide(&mut self) {
        self.root.set_visible(false);
        self.lifetime = None;
    }
}

impl Pickable for FeedbackPresenter {
    fn collider_instance_id(&self) -> InstanceId {
        self.root.instance_id_unchecked()
    }

    fn on_click(&mut self, ctx: &mut PickingContext, _event: &MouseButtonEvent) {
        ctx.cmds.push(UiCommand::TrialStart(self.entity));
    }
}
