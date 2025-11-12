//! Dishrupt Godot Input Listener
//!
//! This module defines a Godot node that listens for input events and queues them for processing.

use std::collections::VecDeque;

use godot::{
    classes::{InputEvent, InputEventKey, InputEventMouseButton, InputEventMouseMotion},
    prelude::*,
};

use crate::event::{GodotInputEvent, KeyEvent, MouseButtonEvent, MouseMotionEvent};

impl GodotInputEvent {
    /// Check if the event corresponds to an action type.
    pub fn is_action_type(&self) -> bool {
        match self {
            GodotInputEvent::Button(e) => e.raw.is_action_type(),
            GodotInputEvent::Motion(e) => e.raw.is_action_type(),
            GodotInputEvent::Key(e) => e.raw.is_action_type(),
        }
    }

    /// Check if the action is currently pressed.
    pub fn is_action_pressed(&self, action: &str) -> bool {
        match self {
            GodotInputEvent::Button(e) => e.raw.is_action_pressed(action),
            GodotInputEvent::Motion(e) => e.raw.is_action_pressed(action),
            GodotInputEvent::Key(e) => e.raw.is_action_pressed(action),
        }
    }

    /// Check if the action is currently pressed, with exact match.
    pub fn is_action_pressed_exact(&self, action: &str) -> bool {
        match self {
            GodotInputEvent::Button(e) => {
                e.raw.is_action_pressed_ex(action).exact_match(true).done()
            }
            GodotInputEvent::Motion(e) => {
                e.raw.is_action_pressed_ex(action).exact_match(true).done()
            }
            GodotInputEvent::Key(e) => e.raw.is_action_pressed_ex(action).exact_match(true).done(),
        }
    }

    /// Check if the action was released.
    pub fn is_action_released(&self, action: &str) -> bool {
        match self {
            GodotInputEvent::Button(e) => e.raw.is_action_released(action),
            GodotInputEvent::Motion(e) => e.raw.is_action_released(action),
            GodotInputEvent::Key(e) => e.is_action_released(action),
        }
    }

    /// Check if the action was released, with exact match.
    pub fn is_action_released_exact(&self, action: &str) -> bool {
        match self {
            GodotInputEvent::Button(e) => {
                e.raw.is_action_released_ex(action).exact_match(true).done()
            }
            GodotInputEvent::Motion(e) => {
                e.raw.is_action_released_ex(action).exact_match(true).done()
            }
            GodotInputEvent::Key(e) => e.is_action_released_exact(action),
        }
    }

    /// Get the mouse position associated with the event.
    pub fn mouse_position(&self) -> Vector2 {
        match self {
            GodotInputEvent::Button(e) => e.position,
            GodotInputEvent::Motion(e) => e.position,
            GodotInputEvent::Key(_) => Default::default(),
        }
    }
}

/// A Godot node that listens for input events and queues them.
#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct InputListener {
    input_event: VecDeque<GodotInputEvent>,

    base: Base<Node>,
}

impl InputListener {
    /// Drain all queued input events.
    pub fn drain_events(&mut self) -> impl Iterator<Item = GodotInputEvent> {
        self.input_event.drain(..)
    }
}

#[godot_api]
impl INode for InputListener {
    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        self.input_event.push_back(match_class! {event,
            e @ InputEventMouseButton => GodotInputEvent::Button(MouseButtonEvent {
                position: e.get_position(),
                button: e.get_button_index(),
                pressed: e.is_pressed(),
                double_click: e.is_double_click(),
                ctrl_key: e.is_ctrl_pressed(),
                shift_key: e.is_shift_pressed(),
                alt_key: e.is_alt_pressed(),
                raw: e,
            }),
            e @ InputEventMouseMotion => GodotInputEvent::Motion(MouseMotionEvent {
                position: e.get_position(),
                ctrl_key: e.is_ctrl_pressed(),
                shift_key: e.is_shift_pressed(),
                alt_key: e.is_alt_pressed(),
                raw: e,
            }),
            e @ InputEventKey => GodotInputEvent::Key(KeyEvent {
                keycode: e.get_keycode(),
                pressed: e.is_pressed(),
                raw: e,
            }),
            _ => return,
        });
    }
}
