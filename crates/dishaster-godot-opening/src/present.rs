//! Presenters for opening animation items

use dishrupt_godot_display::GdNode2D;
use godot::{
    classes::{RichTextLabel, Sprite2D},
    prelude::*,
};

/// Presenter for food emoji - manages sprite frame and color modulation
pub struct DishPresenter {
    sprite: Gd<Sprite2D>,
}

impl DishPresenter {
    pub fn new(node: GdNode2D, variant: u8, color: (f32, f32, f32)) -> Self {
        let mut sprite = node.get_node_as::<Sprite2D>("Sprite2D");

        // Set sprite frame based on variant
        sprite.set_frame(variant as i32);

        // Set color modulation
        sprite.set_modulate(Color::from_rgb(color.0, color.1, color.2));

        Self { sprite }
    }

    /// Update visual effects (alpha, etc.)
    pub fn update(&mut self, alpha: f32) {
        let mut color = self.sprite.get_modulate();
        color.a = alpha;
        self.sprite.set_modulate(color);
    }
}

/// Presenter for face emoji - manages sprite frame
pub struct EmojiPresenter {
    sprite: Gd<Sprite2D>,
}

impl EmojiPresenter {
    pub fn new(node: GdNode2D, variant: u8) -> Self {
        let mut sprite = node.get_node_as::<Sprite2D>("Sprite2D");

        // Set sprite frame based on variant
        sprite.set_frame(variant as i32);

        Self { sprite }
    }

    /// Update visual effects (alpha, etc.)
    pub fn update(&mut self, alpha: f32) {
        let mut color = self.sprite.get_modulate();
        color.a = alpha;
        self.sprite.set_modulate(color);
    }
}

/// Presenter for review text - manages label text content and wave animation
pub struct TextPresenter {
    label: Gd<RichTextLabel>,
    base_position: Vector2,
}

impl TextPresenter {
    pub fn new(node: GdNode2D, content: String) -> Self {
        let mut label = node.get_node_as::<RichTextLabel>("Label");

        // Set text content
        label.set_text(&content);

        let base_position = label.get_position();

        Self {
            label,
            base_position,
        }
    }

    /// Update visual effects (alpha, wave animation, and optional color)
    pub fn update(&mut self, alpha: f32, wave_phase: f32, color: Option<(f32, f32, f32)>) {
        // Update alpha and optionally color
        let mut modulate = self.label.get_modulate();
        modulate.a = alpha;
        if let Some((r, g, b)) = color {
            modulate.r = r;
            modulate.g = g;
            modulate.b = b;
        }
        self.label.set_modulate(modulate);

        // Update wave animation (horizontal sinusoidal motion)
        let wave_amplitude = 10.0; // pixels
        let offset_x = wave_phase.sin() * wave_amplitude;
        self.label.set_position(Vector2::new(
            self.base_position.x + offset_x,
            self.base_position.y,
        ));
    }
}
