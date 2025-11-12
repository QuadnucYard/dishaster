//! Dishrupt Godot Input Events
//!
//!  This module defines structures representing various input events received from Godot.

pub use godot::global::{Key, MouseButton};
use godot::{
    builtin::Vector2,
    classes::{InputEventKey, InputEventMouseButton, InputEventMouseMotion},
    obj::Gd,
};

/// An input event received from Godot.
#[derive(Debug)]
pub enum GodotInputEvent {
    /// Mouse button event.
    Button(MouseButtonEvent),
    /// Mouse motion event.
    Motion(MouseMotionEvent),
    /// Key event.
    Key(KeyEvent),
}

/// Mouse button event received from Godot.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct MouseButtonEvent {
    pub position: Vector2,
    pub button: MouseButton,
    pub pressed: bool,
    pub double_click: bool,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub raw: Gd<InputEventMouseButton>,
}

/// Mouse motion event received from Godot.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct MouseMotionEvent {
    pub position: Vector2,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub raw: Gd<InputEventMouseMotion>,
}

/// Key event received from Godot.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct KeyEvent {
    pub keycode: Key,
    pub raw: Gd<InputEventKey>,
    pub pressed: bool,
}

impl KeyEvent {
    /// Check if the action was released.
    pub fn is_action_released(&self, action: &str) -> bool {
        self.raw.is_action_released(action)
    }

    /// Check if the action was released, with exact match.
    pub fn is_action_released_exact(&self, action: &str) -> bool {
        self.raw
            .is_action_released_ex(action)
            .exact_match(true)
            .done()
    }
}
