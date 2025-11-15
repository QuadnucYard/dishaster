use godot::{classes::Tween, prelude::*};

use crate::prelude::*;

/// Hint notification that appears temporarily to guide the player

#[derive(UITree)]
#[ui_tree]
pub struct HintNotification {
    #[child("%Panel")]
    panel: ControlA,
    #[child("%Label")]
    label: LabelA,

    tween: Option<Gd<Tween>>,
}

#[ui_tree_api]
impl UITree for HintNotification {}

impl Gui for HintNotification {}

impl HintNotification {
    /// Show a hint message with fade-in animation
    pub fn show_hint(&mut self, message: &str) {
        if let Some(mut tween) = self.tween.take() {
            tween.kill();
        }

        self.label.set_text(message);

        self.show();

        // Animate fade-in
        self.panel.set_modulate(Color::TRANSPARENT_WHITE);
        let mut tween = self.panel.gd().create_tween().unwrap();
        tween.tween_property(&self.panel.gd(), "modulate:a", &1.0.to_variant(), 0.3);
        tween.tween_interval(3.0);
        tween.tween_property(&self.panel.gd(), "modulate:a", &0.0.to_variant(), 0.3);

        self.tween = Some(tween);
    }
}
