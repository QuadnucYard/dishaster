use godot::classes::{Texture2D, TextureRect};

use super::{ControlA, prelude::*};

#[ui_element(TextureRect, base = ControlA)]
pub struct TextureRectA {}

impl TextureRectA {
    pub fn new(gd: Gd<TextureRect>) -> Self {
        Self {
            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }

    pub fn set_texture(&mut self, texture: Gd<Texture2D>) {
        self.gd.set_texture(&texture);
    }

    pub fn set_texture_opt(&mut self, texture: Option<Gd<Texture2D>>) {
        if let Some(texture) = texture {
            self.gd.set_texture(&texture);
        } else {
            self.gd.set_texture(Gd::null_arg());
        }
    }
}
