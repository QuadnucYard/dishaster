use dishaster_views::ReputationView;
use godot::{
    classes::{Tween, tween},
    prelude::*,
};

use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct ReputationGui {
    #[child("%ReputationRow/Value")]
    value_label: LabelA,
    #[child("%ReputationRow/Delta")]
    delta_label: LabelA,
    #[child("%ReputationBar")]
    reputation_bar: ProgressBarA,
    #[child("%FSRIRow/Value")]
    fsri_label: LabelA,
    #[child("%FoodQualityRow/Value")]
    food_quality_label: LabelA,

    /// Active tween for animations (if any)
    active_tween: Option<Gd<Tween>>,
}

impl ReputationGui {
    /// Update reputation display with current values and changes.
    pub fn update(&mut self, view: &ReputationView) {
        // Kill any existing tween
        if let Some(mut tween) = self.active_tween.take() {
            tween.kill();
        }

        // Update reputation value (instant text update)
        self.value_label
            .set_text(&format!("{:.1}", view.reputation));

        // Animate reputation bar
        self.tween_bar(view.reputation);

        // Handle delta label with animation
        if view.reputation_delta.abs() > 0.01 {
            self.tween_delta_label(view.reputation_delta);
        } else {
            self.delta_label.set_visible(false);
        }

        // Update FSRI
        self.fsri_label.set_text(&format!("{:.1}", view.fsri));

        // Update Food Quality
        self.food_quality_label
            .set_text(&format!("{:.1}", view.food_quality));
    }

    fn tween_bar(&mut self, reputation: f32) {
        // Animate progress bar value change
        let mut bar_tween = self.reputation_bar.gd().create_tween();
        bar_tween.set_ease(tween::EaseType::OUT);
        bar_tween.set_trans(tween::TransitionType::CUBIC);
        bar_tween.tween_property(
            &self.reputation_bar.gd(),
            "value",
            &reputation.to_variant(),
            0.5,
        );

        // Color code the bar based on reputation level (animate color transition)
        let target_color = if reputation >= 70.0 {
            Color::from_rgb(0.2, 0.8, 0.2) // High: green
        } else if reputation >= 40.0 {
            Color::from_rgb(0.9, 0.7, 0.1) // Medium: yellow/orange
        } else {
            Color::from_rgb(0.9, 0.2, 0.2) // Low: red
        };

        bar_tween.parallel();
        bar_tween.tween_property(
            &self.reputation_bar.gd(),
            "self_modulate",
            &target_color.to_variant(),
            0.5,
        );
    }

    fn tween_delta_label(&mut self, delta: f32) {
        let delta_text = if delta > 0.0 {
            format!("+{:.1}", delta)
        } else {
            format!("{:.1}", delta)
        };
        self.delta_label.set_text(&delta_text);

        // Color code: green for positive, red for negative
        let delta_color = if delta > 0.0 {
            Color::from_rgb(0.0, 0.8, 0.0)
        } else {
            Color::from_rgb(0.9, 0.1, 0.1)
        };
        self.delta_label.set_self_modulate(delta_color);

        // Animate delta label: scale pulse + fade in, hold, then fade out
        self.delta_label.set_visible(true);
        self.delta_label.set_scale(Vector2::new(0.5, 0.5));
        self.delta_label.set_modulate(Color::WHITE); // Reset alpha to 1.0

        let mut delta_tween = self.delta_label.gd().create_tween();
        delta_tween.set_ease(tween::EaseType::OUT); // Scale up with bounce
        delta_tween.set_trans(tween::TransitionType::BACK);
        delta_tween.tween_property(
            &self.delta_label.gd(),
            "scale",
            &Vector2::new(1.2, 1.2).to_variant(),
            0.3,
        );
        delta_tween.tween_property(
            &self.delta_label.gd(),
            "scale",
            &Vector2::new(1.0, 1.0).to_variant(),
            0.2,
        ); // Scale back to normal
        delta_tween.tween_interval(2.0); // Hold for a moment
        delta_tween.tween_property(&self.delta_label.gd(), "modulate:a", &0.0.to_variant(), 0.5); // Fade out

        // Hide when done using a callback
        let label_gd = self.delta_label.gd();
        delta_tween.tween_callback(&label_gd.callable("set_visible").bind(&[false.to_variant()]));

        self.active_tween = Some(delta_tween);
    }
}

#[ui_tree_api]
impl UITree for ReputationGui {}

impl Gui for ReputationGui {
    fn start(&mut self, _commands: GuiCommands, _provider: AssetProvider) {
        self.delta_label.set_visible(false);
    }
}
