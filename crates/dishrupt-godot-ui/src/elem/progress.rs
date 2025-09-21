use godot::classes::ProgressBar;

use super::{ControlA, prelude::*};

#[ui_element(ProgressBar, base = ControlA)]
pub struct ProgressBarA {}

impl ProgressBarA {
    pub fn new(gd: Gd<ProgressBar>) -> Self {
        Self {
            base: ControlA::new(gd.clone().upcast()),
            gd,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.gd.set_value(value as f64);
    }

    pub fn get_value(&self) -> f32 {
        self.gd.get_value() as f32
    }

    pub fn set_min(&mut self, min: f32) {
        self.gd.set_min(min as f64);
    }

    pub fn get_min(&self) -> f32 {
        self.gd.get_min() as f32
    }

    pub fn set_max(&mut self, max: f32) {
        self.gd.set_max(max as f64);
    }

    pub fn get_max(&self) -> f32 {
        self.gd.get_max() as f32
    }
}
