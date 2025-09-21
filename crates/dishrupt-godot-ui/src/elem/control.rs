use godot::{classes::Control, prelude::*};

use super::prelude::*;

#[ui_element(Control)]
pub struct ControlA {}

impl ControlA {
    pub fn new(gd: Gd<Control>) -> Self {
        Self { gd }
    }

    pub fn free(&mut self) {
        if self.gd.is_instance_valid() {
            self.gd.queue_free()
        }
    }

    pub fn detach(&mut self) {
        if self.gd.is_instance_valid() {
            self.gd.get_parent().inspect(|p| {
                if p.is_instance_valid() {
                    p.clone().remove_child(&self.gd);
                }
            });
        }
    }

    pub fn add_child(&mut self, child: &impl UIElement) {
        self.gd.add_child(&child.gd())
    }

    pub fn move_child(&mut self, child: &impl UIElement, to_index: i32) {
        self.gd.move_child(&child.gd(), to_index);
    }

    pub fn get_node_as<T>(&self, path: &str) -> Gd<T>
    where
        T: Inherits<Node>,
    {
        self.gd.get_node_as(path)
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.gd.set_visible(visible)
    }

    pub fn is_visible(&mut self) -> bool {
        self.gd.is_visible()
    }

    pub fn set_modulate(&mut self, modulate: Color) {
        self.gd.set_modulate(modulate)
    }

    pub fn set_self_modulate(&mut self, self_modulate: Color) {
        self.gd.set_self_modulate(self_modulate)
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.gd.set_position(position)
    }

    pub fn get_position(&mut self) -> Vector2 {
        self.gd.get_position()
    }

    pub fn set_scale(&mut self, position: Vector2) {
        self.gd.set_scale(position)
    }

    pub fn get_scale(&mut self) -> Vector2 {
        self.gd.get_scale()
    }

    pub fn set_size(&mut self, position: Vector2) {
        self.gd.set_size(position)
    }

    pub fn get_size(&mut self) -> Vector2 {
        self.gd.get_size()
    }

    pub fn set_tooltip_text(&mut self, text: &str) {
        self.gd.set_tooltip_text(text)
    }
}
