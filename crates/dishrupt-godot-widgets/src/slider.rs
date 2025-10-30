use godot::classes::Slider;

use super::{ControlA, prelude::*};

#[ui_element(Slider, base = ControlA)]
pub struct SliderA {
    pub on_value_change: Signal<(f32,)>,
    pub on_drag_end: Signal<(Option<f32>,)>,
}

impl SliderA {
    pub fn new(gd: Gd<Slider>) -> Self {
        let on_value_change: Signal<(f32,)> = Signal::new();
        let on_value_change_weak = on_value_change.get_emit_handle();
        gd.signals().value_changed().connect(move |value| {
            on_value_change_weak.emit(value as f32);
        });

        let on_drag_end: Signal<(Option<f32>,)> = Signal::new();
        let on_drag_end_weak = on_drag_end.get_emit_handle();
        let gd_clone = gd.clone();
        gd.signals().drag_ended().connect(move |changed| {
            let v = if changed {
                Some(gd_clone.get_value() as f32)
            } else {
                None
            };
            on_drag_end_weak.emit(v);
        });

        Self {
            on_value_change,
            on_drag_end,

            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }

    pub fn get_value(&self) -> f32 {
        self.gd.get_value() as f32
    }

    pub fn set_value(&mut self, value: f32) {
        self.gd.set_value(value as f64)
    }

    pub fn set_editable(&mut self, editable: bool) {
        self.gd.set_editable(editable);
    }
}
