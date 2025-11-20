use dishaster_interface::event::DinerItemsChange;
use dishaster_views::{Appearance, BodyPart};
use dishrupt_core::EntityId;
use dishrupt_godot_display::{GdNode2D, Stage};
use dishrupt_godot_utils::AnimationPlayerExt;
use godot::{
    classes::{AnimationPlayer, CanvasItem, Label, Node2D, ProgressBar, Sprite2D},
    prelude::*,
};

use super::feedback::FeedbackPresenter;
use crate::present::diner_items::DinerItemsPresenter;

#[allow(unused)]
pub struct AgentPresenter {
    entity: EntityId,

    root: GdNode2D,
    body: Option<GdNode2D>,

    anim_player: Option<Gd<AnimationPlayer>>,

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

        let anim_player = body
            .as_ref()
            .and_then(|body| body.try_get_node_as("AnimationPlayer"));

        Self {
            entity,
            body,

            anim_player,

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

    pub fn update_debug(&mut self, goal_str: &str, total_weight: f32, remaining_weight: f32) {
        if let Some(debug) = &mut self.debug {
            debug.goal_label.set_text(goal_str);

            // Update eating progress bar based on weight data
            if total_weight > 0.0 && remaining_weight > 0.0 {
                debug.update_eating_progress(total_weight, remaining_weight);
            } else {
                debug.hide_eating_progress();
            }
        }
    }

    /// Handle incremental changes to diner items
    pub fn handle_item_change(&mut self, change: DinerItemsChange, stage: &mut Stage) {
        if let Some(anim_player) = &mut self.anim_player {
            match &change {
                DinerItemsChange::StartEating(_, _) => {
                    anim_player.play_by_name("eat");
                }
                DinerItemsChange::FinishEating => {
                    anim_player.play_by_name("walk");
                }
                _ => {}
            }
        }

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

        if let Some(anim_player) = &mut self.anim_player {
            anim_player.set_speed_scale(godot::global::randf_range(1.5, 2.5) as f32);
            anim_player.play_by_name("walk");
        }
    }

    fn apply_to_sprite(body: &GdNode2D, sprite_name: &str, body_part: &BodyPart) {
        let Some(mut sprite) = body.try_get_node_as::<CanvasItem>(sprite_name) else {
            return;
        };

        // Set shader parameters from ColorTransform (now per-part)
        let ct = &body_part.color_transform;
        sprite.set_instance_shader_parameter("hue_shift", &ct.hue_shift.to_variant());
        sprite.set_instance_shader_parameter("saturation", &ct.saturation.to_variant());
        sprite.set_instance_shader_parameter("value", &ct.value.to_variant());
        sprite.set_instance_shader_parameter("alpha", &ct.alpha.to_variant());

        // Set sprite frame to variant index
        if let Ok(mut sprite) = sprite.try_cast::<Sprite2D>() {
            let variant_index = body_part.variant.index() as i32;
            let max_variants = sprite.get_hframes() * sprite.get_vframes();
            if variant_index < max_variants {
                sprite.set_frame(variant_index);
            } else {
                godot_warn!(
                    "Sprite '{}' variant index {} out of bounds (max {})",
                    sprite_name,
                    variant_index,
                    max_variants - 1
                );
            }
        }
    }
}

pub struct AgentDebugPresenter {
    // root: Gd<Node2D>,
    goal_label: Gd<Label>,
    eating_progress: Option<Gd<ProgressBar>>,
}

impl AgentDebugPresenter {
    pub fn new(node: Gd<Node2D>) -> Self {
        let mut eating_progress = node.try_get_node_as::<ProgressBar>("EatingProgress");
        if let Some(progress_bar) = &mut eating_progress {
            progress_bar.set_visible(false);
        }

        Self {
            goal_label: node.get_node_as("GoalLabel"),
            eating_progress,
            // root: node,
        }
    }

    /// Update eating progress bar with current eating state
    pub fn update_eating_progress(&mut self, total_weight: f32, remaining_weight: f32) {
        if let Some(progress_bar) = &mut self.eating_progress {
            if total_weight > 0.0 {
                // Calculate progress as percentage eaten (1.0 - remaining/total)
                let eaten_ratio = (total_weight - remaining_weight) / total_weight;
                progress_bar.set_value(eaten_ratio.clamp(0.0, 1.0) as f64);
                progress_bar.set_visible(true);
            } else {
                // No food or finished eating
                progress_bar.set_visible(false);
            }
        }
    }

    /// Hide the eating progress bar
    pub fn hide_eating_progress(&mut self) {
        if let Some(progress_bar) = &mut self.eating_progress {
            progress_bar.set_visible(false);
        }
    }
}
