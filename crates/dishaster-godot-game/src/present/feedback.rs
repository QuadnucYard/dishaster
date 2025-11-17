use dishaster_ui_protocol::UiCommand;
use dishaster_views::{Feedback, FeedbackTopic};
use dishrupt_core::EntityId;
use dishrupt_godot_input::event::MouseButtonEvent;
use dishrupt_godot_utils::AnimationPlayerExt;
use godot::{
    classes::{AnimationPlayer, Label, Node2D},
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
    anim_player: Gd<AnimationPlayer>,

    lifetime: Option<f64>,

    /// Topic associated with the current feedback (if any).
    topic: Option<FeedbackTopic>,
    /// Whether the current feedback can trigger a trial.
    can_trigger_trial: bool,
}

impl FeedbackPresenter {
    pub fn new(entity: EntityId, node: Gd<Node2D>) -> Self {
        Self {
            entity,

            thought: node.get_node_as("Thought"),
            thought_emoji: node.get_node_as("Thought/Emoji"),
            speech: node.get_node_as("Speech"),
            anim_player: node.get_node_as("%AnimationPlayer"),
            root: node,

            lifetime: None,
            topic: None,
            can_trigger_trial: false,
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

            if self.can_trigger_trial {
                self.anim_player.stop();
            }
        }
    }

    pub fn show(&mut self, feedback: &Feedback, topic: Option<FeedbackTopic>, can_trigger: bool) {
        /// How long feedback balloons last on screen (placeholder value).
        const BALLOON_LIFETIME: f64 = 3.0;

        self.topic = topic;
        self.can_trigger_trial = can_trigger;

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

        if can_trigger {
            self.root.set_modulate(Color::WHITE.with_alpha(1.0));
            self.anim_player.play_by_name("bounce");
        } else {
            self.root.set_modulate(Color::WHITE.with_alpha(0.8));
        }
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

    fn is_active(&self) -> bool {
        self.can_trigger_trial
    }

    fn on_click(&mut self, ctx: &mut PickingContext, _event: &MouseButtonEvent) {
        // Only trigger trial if this feedback can do so
        if self.can_trigger_trial {
            ctx.cmds.push(UiCommand::TrialStart {
                diner: self.entity,
                topic: self.topic,
            });
        }
    }
}
