use std::collections::VecDeque;

use godot::{
    classes::{InputEvent, InputEventKey, InputEventMouseButton, InputEventMouseMotion},
    prelude::*,
};

use super::event::{KeyEvent, MouseButtonEvent, MouseMotionEvent};

#[derive(Debug)]
pub enum GodotInputEvent {
    Button(MouseButtonEvent),
    Motion(MouseMotionEvent),
    Key(KeyEvent),
}

impl GodotInputEvent {
    pub fn is_action_type(&self) -> bool {
        match self {
            GodotInputEvent::Button(e) => e.raw.is_action_type(),
            GodotInputEvent::Motion(e) => e.raw.is_action_type(),
            GodotInputEvent::Key(e) => e.raw.is_action_type(),
        }
    }

    pub fn is_action_pressed(&self, action: &str) -> bool {
        match self {
            GodotInputEvent::Button(e) => e.raw.is_action_pressed(action),
            GodotInputEvent::Motion(e) => e.raw.is_action_pressed(action),
            GodotInputEvent::Key(e) => e.raw.is_action_pressed(action),
        }
    }

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

    pub fn is_action_released(&self, action: &str) -> bool {
        match self {
            GodotInputEvent::Button(e) => e.raw.is_action_released(action),
            GodotInputEvent::Motion(e) => e.raw.is_action_released(action),
            GodotInputEvent::Key(e) => e.is_action_released(action),
        }
    }

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

    pub fn mouse_position(&self) -> Vector2 {
        match self {
            GodotInputEvent::Button(e) => e.position,
            GodotInputEvent::Motion(e) => e.position,
            GodotInputEvent::Key(_) => Default::default(),
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct InputListener {
    input_event: VecDeque<GodotInputEvent>,

    base: Base<Node>,
}

impl InputListener {
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
