use dishaster_views::TrialImpactView;
use dishrupt_godot_utils::AnimationPlayerExt;
use godot::{
    classes::{AnimationPlayer, Label},
    prelude::*,
};

use crate::prelude::*;

const IMPACT_DISPLAY_DURATION: f32 = 3.0; // How long to show impact feedback

#[derive(UITree)]
#[ui_tree]
pub struct TrialImpactGui {
    #[child("%ImpactContainer")]
    impact_container: ControlA,

    #[child("%ImpactLabelTemplate")]
    impact_label_template: ControlA,

    /// Pending impact to display
    pending_impact: Option<ImpactDisplayState>,
}

/// State for displaying trial impact feedback
#[derive(Debug, Clone)]
struct ImpactDisplayState {
    /// The impact data to display
    impact: TrialImpactView,
    /// Time elapsed since impact started displaying
    display_time: f32,
    /// Whether the impact has been visually shown
    shown: bool,
}

impl TrialImpactGui {
    /// Display trial impact feedback in the UI
    ///
    /// Shows changes to both diner psychology and reputation resulting from
    /// trial interactions. This provides real-time feedback to the player
    /// about the consequences of their responses.
    pub fn show_impact(&mut self, impact: TrialImpactView) {
        // Store the impact for display during next process cycle
        self.pending_impact = Some(ImpactDisplayState {
            impact,
            display_time: 0.0,
            shown: false,
        });
    }

    /// Process and display the pending impact feedback
    fn process_impact_display(&mut self, delta: f32) {
        // Check if we need to display impact
        if let Some(mut impact_state) = self.pending_impact.take() {
            if !impact_state.shown {
                self.display_impact_visuals(&impact_state.impact);
                impact_state.shown = true;
            }

            // Update display time and check for removal
            impact_state.display_time += delta;
            if impact_state.display_time < IMPACT_DISPLAY_DURATION {
                self.pending_impact = Some(impact_state);
            }
        }
    }

    /// Display visual feedback for trial impacts
    ///
    /// Creates floating text labels showing changes to diner psychology and reputation.
    /// Labels are color-coded and animated for clear visual feedback.
    fn display_impact_visuals(&mut self, impact: &TrialImpactView) {
        godot_print!("Displaying trial impact visuals: {:?}", impact);

        let mut impact_messages = Vec::new();

        if let Some(psych) = &impact.psych_impact {
            if psych.mood_delta.abs() > 0.01 {
                let mood_emoji = if psych.mood_delta > 0.0 {
                    "😊"
                } else {
                    "😟"
                };
                impact_messages.push((
                    format!("{} Mood: {:+.1}%", mood_emoji, psych.mood_delta * 100.0),
                    psych.mood_delta > 0.0,
                ));
            }

            if psych.trust_delta.abs() > 0.01 {
                let trust_emoji = if psych.trust_delta > 0.0 {
                    "🤝"
                } else {
                    "💔"
                };
                impact_messages.push((
                    format!("{} Trust: {:+.1}%", trust_emoji, psych.trust_delta * 100.0),
                    psych.trust_delta > 0.0,
                ));
            }

            if psych.patience_delta.abs() > 0.1 {
                let patience_emoji = if psych.patience_delta > 0.0 {
                    "⏰"
                } else {
                    "⏱️"
                };
                impact_messages.push((
                    format!("{} Patience: {:+.1}s", patience_emoji, psych.patience_delta),
                    psych.patience_delta > 0.0,
                ));
            }
        }

        if let Some(rep) = &impact.reputation_impact
            && rep.reputation_delta.abs() > 0.01
        {
            let rep_emoji = if rep.reputation_delta > 0.0 {
                "⭐"
            } else {
                "📉"
            };
            impact_messages.push((
                format!("{} Reputation: {:+.1}", rep_emoji, rep.reputation_delta),
                rep.reputation_delta > 0.0,
            ));
        }

        godot_print!("Impact messages to display: {:?}", impact_messages);

        // Create and display floating labels for each impact
        for (message, is_positive) in impact_messages {
            self.create_impact_label(&message, is_positive);
        }
    }

    /// Create a single impact label with animation
    fn create_impact_label(&mut self, text: &str, is_positive: bool) {
        // Duplicate the template
        let mut label_instance = self.impact_label_template.dup().gd();

        // Get the text label inside and set the message
        // Note: The label is nested inside MarginContainer
        if let Some(mut text_label) = label_instance.try_get_node_as::<Label>("ImpactLabelText") {
            text_label.set_text(text);
        } else {
            godot_error!("Failed to find ImpactLabelText node in template");
        }

        // Style based on impact type
        let bg_color = if is_positive {
            Color::from_rgba(0.2, 0.8, 0.3, 0.6) // Green for positive
        } else {
            Color::from_rgba(0.9, 0.2, 0.2, 0.6) // Red for negative
        };
        let border_color =
            Color::from_rgba(bg_color.r * 1.2, bg_color.g * 1.2, bg_color.b * 1.2, 1.0);

        label_instance.add_theme_color_override("bg_color", bg_color);
        label_instance.add_theme_color_override("border_color", border_color);
        label_instance.set_visible(true);

        let mut anim_player = label_instance.get_node_as::<AnimationPlayer>("AnimationPlayer");
        anim_player.play_by_name("in");

        // Add to container
        self.impact_container.gd().add_child(&label_instance);
    }
}

#[ui_tree_api]
impl UITree for TrialImpactGui {}

impl Gui for TrialImpactGui {
    fn start(&mut self, _commands: GuiCommands) {
        self.impact_label_template.set_visible(false);
    }

    fn process(&mut self, _cmd: GuiCommands, delta: f64) {
        let delta = delta as f32;

        // Process impact display regardless of phase
        self.process_impact_display(delta);
    }
}
