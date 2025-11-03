use dishaster_views::{Appearance, BodyPart};
use dishrupt_core::EntityId;
use dishrupt_godot::display::*;
use godot::{
    classes::{CanvasItem, Label, Node2D},
    prelude::*,
};

use super::feedback::FeedbackPresenter;

#[allow(unused)]
pub struct AgentPresenter {
    entity: EntityId,

    root: GdNode2D,
    body: Option<GdNode2D>,
    pub feedback: FeedbackPresenter,
    debug: Option<AgentDebugPresenter>,

    // Slot nodes for item attachment
    tray_slot: Option<Gd<Node2D>>,
    chopsticks_slot: Option<Gd<Node2D>>,
}

impl AgentPresenter {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        let mut feedback = FeedbackPresenter::new(entity, node.get_node_as("Feedback"));
        feedback.hide();

        let debug = node.try_get_node_as("Debug").map(AgentDebugPresenter::new);
        let body = node.try_get_node_as::<Node2D>("Body").map(GdNode2D::new);

        // Try to get slot nodes (attachment points for items)
        let tray_slot = node.try_get_node_as::<Node2D>("Body/TraySlot");
        let chopsticks_slot = node.try_get_node_as::<Node2D>("Body/ChopsticksSlot");

        Self {
            entity,
            body,
            feedback,
            root: node,
            debug,
            tray_slot,
            chopsticks_slot,
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

    /// Update item indicators based on what the diner is carrying
    pub fn update_items(
        &mut self,
        is_eating: bool,
        tray_entity: Option<EntityId>,
        chopsticks_entity: Option<EntityId>,
        stage: &mut Stage,
    ) {
        // Update tray visibility and attachment
        if let Some(tray_id) = tray_entity
            && let Some(tray_node) = stage.get_godot_node_mut(tray_id)
            && !is_eating
        {
            // Attach to tray slot and show
            if let Some(slot) = &self.tray_slot {
                tray_node.reparent(slot); // todo: check parent
                tray_node.set_position(Vector2::ZERO);
            } else {
                godot_warn!(
                    "Agent entity {:?} is missing tray slot for tray entity {:?}",
                    self.entity,
                    tray_id
                );
            }
        }

        // Update chopsticks visibility and attachment
        if let Some(chopsticks_id) = chopsticks_entity
            && let Some(chopsticks_node) = stage.get_godot_node(chopsticks_id)
        {
            let mut chopsticks_node = chopsticks_node.clone();

            if !is_eating {
                // Reparent based on whether tray exists
                if let Some(tray_id) = tray_entity
                    && let Some(tray_node) = stage.get_godot_node(tray_id)
                {
                    // Attach to tray as child
                    chopsticks_node.reparent(&**tray_node);
                    chopsticks_node.set_position(Vector2::new(0.0, -5.0)); // Offset on tray
                } else if let Some(slot) = &self.chopsticks_slot {
                    // Attach to chopsticks slot
                    chopsticks_node.reparent(slot);
                    chopsticks_node.set_position(Vector2::ZERO);
                } else {
                    godot_warn!(
                        "Agent entity {:?} is missing chopsticks slot for chopsticks entity {:?}",
                        self.entity,
                        chopsticks_id
                    );
                }
            }
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

pub struct AgentDebugPresenter {
    // root: Gd<Node2D>,
    goal_label: Gd<Label>,
}

impl AgentDebugPresenter {
    pub fn new(node: Gd<Node2D>) -> Self {
        Self {
            goal_label: node.get_node_as("GoalLabel"),
            // root: node,
        }
    }
}
