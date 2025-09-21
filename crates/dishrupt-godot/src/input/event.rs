pub use godot::global::{Key, MouseButton};
use godot::{
    builtin::Vector2,
    classes::{InputEventKey, InputEventMouseButton, InputEventMouseMotion},
    obj::Gd,
};

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

pub struct MouseMotionEvent {
    pub position: Vector2,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub raw: Gd<InputEventMouseMotion>,
}

pub struct KeyEvent {
    pub keycode: Key,
    pub raw: Gd<InputEventKey>,
}

impl KeyEvent {
    pub fn is_action_released(&self, action: &str) -> bool {
        self.raw.is_action_released(action)
    }

    pub fn is_action_released_exact(&self, action: &str) -> bool {
        self.raw
            .is_action_released_ex(action)
            .exact_match(true)
            .done()
    }
}
