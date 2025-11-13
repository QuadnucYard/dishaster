use dishaster_views::{Feedback, FeedbackView};

use super::prelude::*;

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
) {
    for msg in feedback_messages.read() {
        events.push(SimEvent::Feedback(FeedbackView {
            entity: msg.entity.to_entity_id(),
            content: msg.content.clone(),
        }));

        // Apply reputation impact if feedback has a topic
        if let Some(topic) = msg.trigger {
            let base_impact = reputation_config.base_impacts.get(topic);
            // Use 0.0 response_score for non-trial feedback (neutral player response)
            // Trial system will handle response scoring separately
            reputation.apply_feedback_impact(base_impact, 0.0, &reputation_config);
        }
    }
}
