use dishaster_ui_protocol::UiCommand;
use dishrupt_core::EntityId;
use dishrupt_godot_display::GdNode2D;
use dishrupt_godot_input::event::MouseButtonEvent;
use godot::{
    classes::{Area2D, Sprite2D},
    prelude::*,
};

use crate::input::{Pickable, PickingContext};

// TODO: improve animation

/// Stock warning thresholds
const LOW_STOCK_THRESHOLD: f32 = 0.2;
const MEDIUM_STOCK_THRESHOLD: f32 = 0.5;

/// Animation parameters for low stock warning
const WARNING_PULSE_SPEED: f32 = 3.0; // cycles per second
const WARNING_PULSE_INTENSITY: f32 = 0.4; // modulation amplitude
const WARNING_SHAKE_AMPLITUDE: f32 = 2.0; // pixels

pub struct DispenserPresenter {
    entity: EntityId,
    root: GdNode2D,
    area: Gd<Area2D>,
    sprites: Vec<Gd<Sprite2D>>,
    /// Current stock ratio (0.0 to 1.0)
    stock_ratio: f32,
    /// Animation timer for pulsing effect
    animation_time: f32,
    /// Original sprite positions for shake animation
    sprite_original_positions: Vec<Vector2>,
}

impl DispenserPresenter {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        // Get the Area2D child for collision detection
        let area = node
            .try_get_node_as::<Area2D>("Area2D")
            .expect("Dispenser prefab must have Area2D child");

        // Collect all Sprite2D children for visual effects
        let mut sprites = Vec::new();
        let mut sprite_original_positions = Vec::new();

        for i in 0..node.get_child_count() {
            if let Some(child) = node.get_child(i)
                && let Ok(sprite) = child.try_cast::<Sprite2D>()
            {
                sprite_original_positions.push(sprite.get_position());
                sprites.push(sprite);
            }
        }

        Self {
            entity,
            root: node,
            area,
            sprites,
            stock_ratio: 1.0,
            animation_time: 0.0,
            sprite_original_positions,
        }
    }

    pub fn set_stock(&mut self, current: u32, capacity: u32) {
        self.stock_ratio = current as f32 / capacity.max(1) as f32;
        self.update_visuals();
    }

    /// Update visual effects based on current stock level
    fn update_visuals(&mut self) {
        let base_color = if self.stock_ratio < LOW_STOCK_THRESHOLD {
            // Low stock: red tint
            Color::from_rgb(1.0, 0.6, 0.6)
        } else if self.stock_ratio < MEDIUM_STOCK_THRESHOLD {
            // Medium stock: yellow tint
            Color::from_rgb(1.0, 0.95, 0.7)
        } else {
            // Good stock: normal
            Color::from_rgb(1.0, 1.0, 1.0)
        };

        // Apply pulsing animation for low stock warning
        let modulate = if self.stock_ratio < LOW_STOCK_THRESHOLD {
            let pulse = (self.animation_time * WARNING_PULSE_SPEED * std::f32::consts::TAU).sin();
            let intensity = 1.0 - WARNING_PULSE_INTENSITY * (pulse * 0.5 + 0.5);
            Color::from_rgba(
                base_color.r * intensity,
                base_color.g * intensity,
                base_color.b * intensity,
                1.0,
            )
        } else {
            base_color
        };

        self.root.set_modulate(modulate);

        // Apply shake animation to sprites when low on stock
        if self.stock_ratio < LOW_STOCK_THRESHOLD {
            let shake =
                (self.animation_time * WARNING_PULSE_SPEED * 2.0 * std::f32::consts::TAU).sin();
            let offset_x = shake * WARNING_SHAKE_AMPLITUDE;

            for (sprite, original_pos) in self
                .sprites
                .iter_mut()
                .zip(self.sprite_original_positions.iter())
            {
                sprite.set_position(*original_pos + Vector2::new(offset_x, 0.0));
            }
        } else {
            // Reset sprite positions to original
            for (sprite, original_pos) in self
                .sprites
                .iter_mut()
                .zip(self.sprite_original_positions.iter())
            {
                sprite.set_position(*original_pos);
            }
        }
    }

    /// Update animation state (call every frame)
    pub fn process(&mut self, delta: f32) {
        if self.stock_ratio < LOW_STOCK_THRESHOLD {
            self.animation_time += delta;
            self.update_visuals();
        }
    }
}

impl Pickable for DispenserPresenter {
    fn collider_instance_id(&self) -> InstanceId {
        self.area.instance_id_unchecked()
    }

    fn on_click(&mut self, ctx: &mut PickingContext, _event: &MouseButtonEvent) {
        godot_print!("Dispenser clicked: {:?}", self.entity);
        // Send command to request refill
        ctx.cmds.push(UiCommand::RefillDispenser(self.entity));
    }
}
