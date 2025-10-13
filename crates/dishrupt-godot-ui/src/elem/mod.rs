mod button;
mod control;
mod label;
mod line_edit;
mod progress;
mod slider;
mod texture_rect;

pub use button::{ButtonA, TextButtonA, TextureButtonA};
pub use control::ControlA;
use godot::{classes::Control, obj::Gd};
pub use label::{LabelA, RichLabelA};
pub use line_edit::LineEditA;
pub use progress::ProgressBarA;
pub use slider::SliderA;
pub use texture_rect::TextureRectA;

mod prelude {
    pub use dishrupt_godot_ui_macros::ui_element;
    pub use godot::obj::Gd;
    pub use signals2::*;

    pub use super::UIElement;

    pub trait ObjectExt {
        fn clear_connections(&mut self, signal: &str);
    }

    impl ObjectExt for godot::classes::Object {
        fn clear_connections(&mut self, signal: &str) {
            for conn in self.get_signal_connection_list(signal).iter_shared() {
                self.disconnect(
                    &conn.at("signal").to::<godot::builtin::Signal>().name(),
                    &conn.at("callable").to(),
                );
            }
        }
    }
}

pub trait UIElement {
    fn gd(&self) -> Gd<Control>;

    fn dup(&self) -> Self;

    fn destroy(&self) {
        let gd = self.gd();
        if gd.is_instance_valid() {
            gd.get_parent().inspect(|p| {
                if p.is_instance_valid() {
                    p.clone().remove_child(&gd);
                }
            });
        }
    }
}
