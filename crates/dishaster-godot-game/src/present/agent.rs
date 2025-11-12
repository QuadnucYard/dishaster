use dishaster_interface::event::DinerItemsChange;
use dishaster_views::{Appearance, BodyPart};
use dishrupt_core::EntityId;
use dishrupt_godot_display::{GdNode2D, Stage};
use godot::{
    classes::{CanvasItem, Label, Node2D},
    prelude::*,
};

use super::feedback::FeedbackPresenter;
use crate::present::diner_items::DinerItemsPresenter;

#[allow(unused)]
pub struct AgentPresenter {
    entity: EntityId,

    root: GdNode2D,
    body: Option<GdNode2D>,
    pub feedback: Option<FeedbackPresenter>,
    debug: Option<AgentDebugPresenter>,

    // Current item state
    diner_items: Option<DinerItemsPresenter>,
}

impl AgentPresenter {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        let feedback = node.try_get_node_as("Feedback").map(|node| {
            let mut feedback = FeedbackPresenter::new(entity, node);
            feedback.hide();
            feedback
        });

        let debug = node.try_get_node_as("Debug").map(AgentDebugPresenter::new);
        let body = node.try_get_node_as::<Node2D>("Body").map(GdNode2D::new);

        let diner_items = body.as_ref().and_then(|body| {
            body.try_get_node_as::<Node2D>("ItemSlots")
                .map(|node| DinerItemsPresenter::new(entity, GdNode2D::new(node)))
        });

        Self {
            entity,
            body,
            feedback,
            root: node,
            debug,
            diner_items,
        }
    }

    pub fn set_debug_enabled(&mut self, enabled: bool) {
        if let Some(debug) = &mut self.debug {
            debug.goal_label.set_visible(enabled);
        }
    }

    pub fn process(&mut self, delta: f64, stage: &mut Stage) {
        if let Some(diner_items) = &mut self.diner_items {
            diner_items.process(delta, stage);
        }
        if let Some(feedback) = &mut self.feedback {
            feedback.process(delta);
        }
    }

    pub fn update_debug(&mut self, goal_str: &str) {
        if let Some(debug) = &mut self.debug {
            debug.goal_label.set_text(goal_str);
        }
    }

    /// Handle incremental changes to diner items
    pub fn handle_item_change(&mut self, change: DinerItemsChange, stage: &mut Stage) {
        if let Some(diner_items) = &mut self.diner_items {
            diner_items.handle_item_change(change, stage);
        } else {
            godot_warn!(
                "Agent entity {:?} has no DinerItemsPresenter to handle item change",
                self.entity,
            );
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
