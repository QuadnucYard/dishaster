use godot::{
    builtin::Variant,
    classes::{Label, RichTextLabel},
};

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
pub struct RichLabelA {
    pub on_meta_click: Signal<(Variant,)>,
}

impl RichLabelA {
    pub fn new(gd: Gd<RichTextLabel>) -> Self {
        let on_meta_click: Signal<(Variant,)> = Signal::new();

        let on_meta_click_handle = on_meta_click.get_emit_handle();
        gd.signals().meta_clicked().connect(move |meta| {
            on_meta_click_handle.emit(meta);
        });

        Self {
            on_meta_click,
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
