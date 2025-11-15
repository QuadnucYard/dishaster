use dishaster_views::{Feedback, FeedbackView};

use super::prelude::*;
use crate::systems::hint::{HintEmitter, hints};

// For now, we use simple emoji strings as feedback indicators.
pub mod feedbacks {

    // These are converted to graphical balloons in the client.
    pub const OBSERVING: &[&str] = &["👀", "🤔", "📝"];
    pub const DECIDING: &[&str] = &["😋", "😕", "💡"];
    pub const SERVING: &[&str] = &["🍚", "🍲", "👍", "❓"];

    // Complaint feedbacks for different triggers
    pub const NO_APPEALING_DISH: &[&str] = &["😞", "😕", "🤷"];
    pub const QUEUE_TOO_LONG: &[&str] = &["😤", "⏰", "💢"];
    pub const MISSING_TABLEWARE: &[&str] = &["😠", "❓", "🤦"];
    // Reserved for appearance quality checking (not yet implemented)
    #[allow(dead_code)]
    pub const APPEARANCE_MISMATCH: &[&str] = &["🤨", "😟", "👎"];
    pub const CONTAMINATION: &[&str] = &["🤢", "😱", "🤮"];
    pub const BAD_TASTE: &[&str] = &["😖", "😞", "🤢"];
    pub const STILL_HUNGRY: &[&str] = &["😐", "🍚", "😕"];
}

pub fn choose_feedback(rng: &mut impl Rng, pool: &[&str]) -> Feedback {
    Feedback::Thought(pool.choose(rng).cloned().expect("pool is non-empty").into())
}

pub fn feedback_present_system(
    mut feedback_messages: MessageReader<FeedbackMessage>,
    mut events: ResMut<EventQueue>,
    mut reputation: ResMut<ReputationStateRes>,
    reputation_config: Res<ReputationConfigRes>,
    trial_session: Res<TrialSession>,
    registry: Res<GameModelRegistryRes>,
    mut rng: ResMut<WorldRng>,
) {
    for msg in feedback_messages.read() {
        let topic = msg.trigger;

        // Apply probability gate for displaying feedback bubble
        // This reduces visual noise with many diners (e.g., 500/day)
        let should_display = topic.is_none_or(|t| {
            let display_prob = reputation_config.display_probabilities[t];
            rng.random_bool(display_prob as f64)
        });

        if should_display {
            // Check if this feedback can trigger a trial
            let can_trigger_trial = topic.is_some_and(|t| {
                registry
                    .trial
                    .diner_speeches
                    .iter()
                    .any(|speech| speech.topic == Some(t) || speech.topic.is_none())
            });

            events.push(SimEvent::Feedback(FeedbackView {
                entity: msg.entity.to_entity_id(),
                content: msg.content.clone(),
                topic: topic.as_ref().map(ToView::to_view),
                can_trigger_trial,
            }));

            // Emit hint for first-time trial trigger opportunity
            if can_trigger_trial && !trial_session.ever_triggered {
                events.emit_hint(hints::CLICK_FEEDBACK_TO_TRIAL, HintCondition::Always);
            }
        }

        // Apply reputation impact if feedback has a topic
        // This is separate from display - impact can apply even if not shown
        if let Some(t) = topic {
            let impact_prob = reputation_config.impact_probabilities[t];

            // Probability gate for reputation impact
            if rng.random_bool(impact_prob as f64) {
                let base_impact = reputation_config.base_impacts[t];
                // Use 0.0 response_score for non-trial feedback (neutral player response)
                // Trial system will handle response scoring separately
                reputation.apply_feedback_impact(base_impact, 0.0, &reputation_config);
            }
        }
    }
}
