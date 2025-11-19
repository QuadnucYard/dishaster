use dishrupt_core::prelude::EcoString;
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
    state: Option<HintState>,
}

struct HintState {
    timer: f64,
    message: EcoString,
}

#[ui_tree_api]
impl UITree for HintNotification {}

impl Gui for HintNotification {
    fn process(&mut self, _commands: GuiCommands, delta: f64) {
        if let Some(state) = &mut self.state {
            state.timer -= delta;
            if state.timer <= 0.0 {
                self.state = None;
            }
        }
    }
}

impl HintNotification {
    const HINT_DURATION: f64 = 3.0;

    /// Show a hint message with fade-in animation
    pub fn show_hint(&mut self, message: &str) {
        if self
            .state
            .as_ref()
            .is_some_and(|state| state.message == message)
        {
            // Same message is already being shown; do nothing
            return;
        }

        if let Some(mut tween) = self.tween.take() {
            tween.kill();
        }

        self.label.set_text(message);

        self.show();

        // Animate fade-in
        self.panel.set_modulate(Color::TRANSPARENT_WHITE);
        let mut tween = self.panel.gd().create_tween().unwrap();
        tween.tween_property(&self.panel.gd(), "modulate:a", &1.0.to_variant(), 0.3);
        tween.tween_interval(Self::HINT_DURATION);
        tween.tween_property(&self.panel.gd(), "modulate:a", &0.0.to_variant(), 0.3);

        self.tween = Some(tween);
        self.state = Some(HintState {
            timer: Self::HINT_DURATION,
            message: message.into(),
        });
    }
}
