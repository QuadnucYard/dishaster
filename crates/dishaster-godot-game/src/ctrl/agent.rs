use dishaster_godot_ui::req::GameRequest;
use dishaster_interface::event::Feedback;
use dishaster_models::{Appearance, BodyPart};
use dishrupt_core::EntityId;
use dishrupt_godot::{display::*, input::event::MouseButtonEvent};
use dishrupt_godot_scene::SceneContext;
use godot::{
    classes::{CanvasItem, Label, Node2D},
    prelude::*,
};

use crate::input::Pickable;

#[allow(unused)]
pub struct AgentController {
    entity: EntityId,

    root: GdNode2D,
    body: Option<GdNode2D>,
    pub feedback: FeedbackController,
    debug: Option<AgentDebugController>,
}

impl AgentController {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        let mut feedback = FeedbackController::new(entity, node.get_node_as("Feedback"));
        feedback.root.set_visible(false);

        let debug = node.try_get_node_as("Debug").map(AgentDebugController::new);
        let body = node.try_get_node_as::<Node2D>("Body").map(GdNode2D::new);

        Self {
            entity,
            body,
            feedback,
            root: node,
            debug,
        }
    }

    pub fn set_debug_enabled(&mut self, enabled: bool) {
        if let Some(debug) = &mut self.debug {
            debug.goal_label.set_visible(enabled);
        }
    }

    pub fn process(&mut self, delta: f64) {
        self.feedback.process(delta);
    }

    pub fn update_debug(&mut self, goal_str: &str) {
        if let Some(debug) = &mut self.debug {
            debug.goal_label.set_text(goal_str);
        }
    }

    /// Apply cosmetic appearance to agent sprites
    pub fn set_appearance(&mut self, appearance: &Appearance) {
        let Some(body) = &self.body else {
            godot_warn!(
                "Agent entity {:?} is missing Body node for appearance application",
                self.entity
            );
            return;
        };

        // Apply to each body part sprite if it exists
        Self::apply_to_sprite(body, "Head", &appearance.head);
        Self::apply_to_sprite(body, "UpperGarment", &appearance.upper_garment);
        Self::apply_to_sprite(body, "LowerGarment", &appearance.lower_garment);
        Self::apply_to_sprite(body, "LeftHand", &appearance.hands);
        Self::apply_to_sprite(body, "RightHand", &appearance.hands);
        Self::apply_to_sprite(body, "LeftShoe", &appearance.shoes);
        Self::apply_to_sprite(body, "RightShoe", &appearance.shoes);
    }

    fn apply_to_sprite(body: &GdNode2D, sprite_name: &str, body_part: &BodyPart) {
        let Some(mut sprite) = body.try_get_node_as::<CanvasItem>(sprite_name) else {
            return;
        };

        // Load texture based on body part and variant
        // Expected file structure: res://assets/sprites/agents/head_00.tres, head_01.tres, etc.
        // let texture_path = format!(
        //     "res://assets/sprites/agents/{}_{:02}.tres",
        //     sprite_name.to_lowercase(),
        //     body_part.variant.index()
        // );

        // if let Ok(texture) = try_load::<Texture2D>(&texture_path) {
        //     sprite.set_texture(&texture);
        // } else {
        //     godot_warn!(
        //         "Failed to load texture variant {} for {}",
        //         body_part.variant.index(),
        //         sprite_name
        //     );
        // }

        // Set shader parameters from ColorTransform (now per-part)
        let ct = &body_part.color_transform;
        sprite.set_instance_shader_parameter("hue_shift", &ct.hue_shift.to_variant());
        sprite.set_instance_shader_parameter("saturation", &ct.saturation.to_variant());
        sprite.set_instance_shader_parameter("value", &ct.value.to_variant());
        sprite.set_instance_shader_parameter("alpha", &ct.alpha.to_variant());
    }
}

pub struct FeedbackController {
    /// The agent entity this feedback belongs to.
    entity: EntityId,

    root: Gd<Node2D>, // actually Area2D
    thought: Gd<Node2D>,
    thought_emoji: Gd<Label>,
    speech: Gd<Node2D>,

    lifetime: Option<f64>,
}

impl FeedbackController {
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
}

impl Pickable for FeedbackController {
    fn collider_instance_id(&self) -> InstanceId {
        self.root.instance_id_unchecked()
    }

    fn on_click(&mut self, ctx: &mut SceneContext, _event: &MouseButtonEvent) {
        ctx.gui_cmds.push_req(GameRequest::TrialStart(self.entity));
    }
}

pub struct AgentDebugController {
    // root: Gd<Node2D>,
    goal_label: Gd<Label>,
}

impl AgentDebugController {
    pub fn new(node: Gd<Node2D>) -> Self {
        Self {
            goal_label: node.get_node_as("GoalLabel"),
            // root: node,
        }
    }
}
