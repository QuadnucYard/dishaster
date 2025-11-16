use dishaster_trial::{PsychImpact, ReputationImpact};
use dishaster_views::{Feedback, FeedbackView, PsychImpactView, ReputationView, TrialImpactView};

use super::prelude::*;
use crate::{
    events::ApplyTrialImpact,
    systems::hint::{HintEmitter, hints},
};

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

pub fn apply_trial_impact(
    event: On<ApplyTrialImpact>,
    mut diner_query: Query<&mut DinerPsychState>,
    mut reputation: ResMut<ReputationStateRes>,
    reputation_config: Res<ReputationConfigRes>,
    mut events: ResMut<EventQueue>,
) {
    let Ok(mut psych_state) = diner_query.get_mut(event.diner) else {
        return;
    };

    let psych_impact_view = apply_psych_impact(&mut psych_state, &event.psych_impact);
    let reputation_impact = apply_reputation_impact(
        &mut reputation,
        &reputation_config,
        &event.reputation_impact,
    );

    let impact_view = TrialImpactView {
        psych_impact: Some(psych_impact_view),
        reputation_impact: Some(reputation_impact),
    };

    events.push(SimEvent::TrialImpact(impact_view.into()));
}

fn apply_reputation_impact(
    reputation: &mut ReputationState,
    reputation_config: &ReputationConfigRes,
    reputation_impact: &ReputationImpact,
) -> ReputationView {
    let base_impact = reputation_config.base_impacts[FeedbackTopic::Quality];
    let old_reputation = reputation.reputation;
    let response_score = reputation_impact.response_score;

    reputation.apply_feedback_impact(base_impact, response_score, reputation_config);

    let reputation_delta = reputation.reputation - old_reputation;

    log::info!(
        "Trial response impact on reputation: {:.2} (score: {:.2})",
        reputation_delta,
        response_score
    );

    ReputationView {
        reputation: reputation.reputation,
        reputation_delta,
        fsri: reputation.fsri,
        food_quality: reputation.food_quality,
    }
}

fn apply_psych_impact(psych_state: &mut PsychState, psych_impact: &PsychImpact) -> PsychImpactView {
    let old_mood = psych_state.mood;
    let old_trust = psych_state.trust;
    let old_patience = psych_state.patience;

    let PsychImpact {
        mood_change,
        trust_change,
        patience_change,
    } = psych_impact;

    // Apply changes with clamping
    psych_state.mood = (psych_state.mood + mood_change).clamp(-1.0, 1.0);
    psych_state.trust = (psych_state.trust + trust_change).clamp(0.0, 1.0);
    psych_state.patience = (psych_state.patience + patience_change).max(0.0);

    let mood_delta = psych_state.mood - old_mood;
    let trust_delta = psych_state.trust - old_trust;
    let patience_delta = psych_state.patience - old_patience;

    log::info!(
        "Trial impact on diner psych: mood={:+.2} trust={:+.2} patience={:+.2}",
        mood_delta,
        trust_delta,
        patience_delta
    );

    PsychImpactView {
        mood_delta,
        trust_delta,
        patience_delta,
    }
}
