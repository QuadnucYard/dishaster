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
    registry: Res<GameModelRegistryRes>,
) {
    for msg in feedback_messages.read() {
        // Check if this feedback can trigger a trial: if there are any diner speeches with this topic in the corpus
        let can_trigger_trial = msg.trigger.is_some_and(|topic| {
            registry
                .trial
                .diner_speeches
                .iter()
                .any(|speech| speech.topic == Some(topic) || speech.topic.is_none())
        });

        events.push(SimEvent::Feedback(FeedbackView {
            entity: msg.entity.to_entity_id(),
            content: msg.content.clone(),
            topic: msg.trigger.as_ref().map(ToView::to_view),
            can_trigger_trial,
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
