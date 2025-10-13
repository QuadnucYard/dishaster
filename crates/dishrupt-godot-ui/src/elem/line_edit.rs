use godot::classes::LineEdit;

use super::{ControlA, prelude::*};

/// Convenience wrapper around Godot's `LineEdit` for GUI binding.
#[ui_element(LineEdit, base = ControlA)]
pub struct LineEditA {
    pub on_text_change: Signal<(String,)>,
    pub on_text_submit: Signal<(String,)>,
}

impl LineEditA {
    pub fn new(gd: Gd<LineEdit>) -> Self {
        let on_text_change: Signal<(String,)> = Signal::new();
        let on_text_change_handle = on_text_change.get_emit_handle();
        gd.signals().text_changed().connect(move |value| {
            on_text_change_handle.emit(value.to_string());
        });

        let on_text_submit: Signal<(String,)> = Signal::new();
        let on_text_submit_handle = on_text_submit.get_emit_handle();
        gd.signals().text_submitted().connect(move |value| {
            on_text_submit_handle.emit(value.to_string());
        });

        Self {
            on_text_change,
            on_text_submit,
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

    pub fn set_placeholder(&mut self, text: &str) {
        self.gd.set_placeholder(text);
    }

    pub fn grab_focus(&mut self) {
        self.gd.grab_focus();
    }
}

impl Drop for LineEditA {
    fn drop(&mut self) {
        if self.gd.is_instance_valid() {
            self.gd.clear_connections("text_changed");
            self.gd.clear_connections("text_submitted");
        }
    }
}
