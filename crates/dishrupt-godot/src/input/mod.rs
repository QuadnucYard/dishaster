use std::any::Any;

use event::{MouseButtonEvent, MouseMotionEvent};
use godot::{classes::Node, obj::Gd};

pub mod event;
pub mod listener;

#[allow(unused_variables)]
pub trait Selectable: Any {
    fn gd(&self) -> Gd<Node>;

    /// The z-index for sorting order detection in events
    fn z_index(&self) -> i32 {
        0
    }

    fn on_click(&mut self, event: &MouseButtonEvent) {}

    fn on_mouse_down(&mut self, event: &MouseButtonEvent) {}

    fn on_mouse_up(&mut self, event: &MouseButtonEvent) {}

    fn on_mouse_enter(&mut self) {}

    fn on_mouse_leave(&mut self) {}

    fn on_mouse_move(&mut self, event: &MouseMotionEvent) {}

    fn on_drag_start(&mut self, event: &MouseMotionEvent) {}

    fn on_drag_end(&mut self, event: &MouseMotionEvent) {}

    fn on_drag(&mut self, event: &MouseMotionEvent) {}
}
