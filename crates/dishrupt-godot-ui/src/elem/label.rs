use godot::classes::{Label, RichTextLabel};

use super::{ControlA, UIElement, prelude::*};

#[ui_element(Label, base = ControlA)]
pub struct LabelA {}

impl LabelA {
    pub fn new(gd: Gd<Label>) -> Self {
        Self {
            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.gd.set_text(text);
    }

    pub fn get_text(&self) -> String {
        self.gd.get_text().into()
    }
}

#[ui_element(RichTextLabel, base = ControlA)]
pub struct RichLabelA {}

impl RichLabelA {
    pub fn new(gd: Gd<RichTextLabel>) -> Self {
        Self {
            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.gd.set_text(text);
    }

    pub fn get_text(&self) -> String {
        self.gd.get_text().into()
    }

    pub fn clear(&mut self) {
        self.gd.clear();
    }
}
