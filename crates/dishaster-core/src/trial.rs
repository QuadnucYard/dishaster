//! The trial system. It works outside the ECS loop.

use bevy_ecs::system::SystemState;

use crate::{
    models::{TrialCorpus, TrialQARank, TrialSpeechItem},
    prelude::*,
    resources::*,
    sim::Simulation,
    views::{self, TrialIntro, TrialResponseOption, TrialStatement},
};

impl Simulation {
    pub(super) fn create_trial_intro(&mut self) -> TrialIntro {
        let mut trial_session = self.world.resource_mut::<TrialSession>();
        TrialIntro {
            left: random_appearance(&mut trial_session.rng),
            right: random_appearance(&mut trial_session.rng),
        }
    }

    pub(super) fn create_diner_statement(&mut self) -> TrialStatement {
        let mut system_state: SystemState<(Res<GameModelRegistryRes>, ResMut<TrialSession>)> =
            SystemState::new(&mut self.world);
        let (registry, mut trial_session) = system_state.get_mut(&mut self.world);

        // Generate a sequence of related speeches (topic-centered)
        let speech_sequence = generate_speech_sequence(&registry.trial, &mut trial_session);

        log::info!("Generated speech sequence: {:?}", speech_sequence);

        create_diner_statement_with_sequence(speech_sequence, &registry.trial, &mut trial_session)
    }

    pub(super) fn trial_respond(&mut self, resp_corpus_index: usize) -> views::TrialSpeech {
        // Get current question index and response data first
        let (current_question_idx, mut response_score, speech) = {
            let trial_session = self.world.resource::<TrialSession>();
            let registry = self.world.resource::<GameModelRegistryRes>();

            let current_question_idx = trial_session.current_question_index;
            let response = &registry.trial.responses[resp_corpus_index];
            let response_score = response.response_score;
            let speech = response.content.to_view();

            (current_question_idx, response_score, speech)
        };

        // Record the player's response choice
        let mut trial_session = self.world.resource_mut::<TrialSession>();
        trial_session.set_last_response(resp_corpus_index);

        // Contextual evaluation: check if response is relevant to the current question
        // Use QA ranks to measure relevance (higher rank = more relevant)
        if let Some(question_idx) = current_question_idx {
            let registry = self.world.resource::<GameModelRegistryRes>();
            let relevance_penalty =
                calculate_relevance_penalty(question_idx, resp_corpus_index, &registry.trial);

            if relevance_penalty < 0.0 {
                log::info!(
                    "Response {} to question {} is irrelevant (penalty: {:.3})",
                    resp_corpus_index,
                    question_idx,
                    relevance_penalty
                );
                response_score += relevance_penalty; // Apply penalty
            }
        }

        // Apply reputation impact
        let mut system_state: SystemState<(ResMut<ReputationStateRes>, Res<ReputationConfigRes>)> =
            SystemState::new(&mut self.world);
        let (mut reputation, reputation_config) = system_state.get_mut(&mut self.world);

        // Apply a moderate base impact modified by response score (including relevance penalty)
        // Using quality topic (-4.0) as a representative negative feedback
        let base_impact = reputation_config.base_impacts.quality;
        reputation.apply_feedback_impact(base_impact, response_score, &reputation_config);

        speech
    }

    /// After player responds, check if diner should continue with new topic.
    /// This uses AQ ranks to select questions related to the player's last response.
    pub(super) fn trial_should_continue(&mut self) -> bool {
        let mut system_state: SystemState<(Res<GameModelRegistryRes>, ResMut<TrialSession>)> =
            SystemState::new(&mut self.world);
        let (registry, mut trial_session) = system_state.get_mut(&mut self.world);

        // Check if there are available follow-up questions based on player's response
        if let Some(last_response_idx) = trial_session.last_response_index
            && let Some(aq_rank) = registry.trial.aq_ranks.get(last_response_idx)
        {
            // Filter available questions (not yet asked)
            let available_ranks: Vec<_> = aq_rank
                .iter()
                .filter(|rank| !trial_session.has_asked(rank.answer_index))
                .cloned()
                .collect();

            if available_ranks.is_empty() {
                return false;
            }
            // Use best score as probability to continue
            let best_score = available_ranks[0].score;
            trial_session.should_continue(best_score)
        } else {
            // No response history or ranks - randomly decide
            trial_session.rng.random_bool(0.3)
        }
    }
}

fn random_appearance(rng: &mut impl Rng) -> views::TrialParticipantAppearance {
    views::TrialParticipantAppearance {
        emotion: [
            '😅', '😡', '😠', '😤', '😞', '😢', '😭', '😰', '😨', '😱', '😠',
        ]
        .choose(rng)
        .copied()
        .unwrap(),
        gesture: [
            '👍', '👎', '👊', '🤚', '✋', '👋', '🤞', '🤏', '👈', '👉', '🤝', '👍', '👏',
        ]
        .choose(rng)
        .copied(),
    }
}

/// Generate a sequence of related speeches using QQ ranks for topic coherence
///
/// This creates multi-turn diner dialogue by:
/// 1. Selecting an initial question using AQ ranks or random selection
/// 2. Following QQ continuation chains based on semantic similarity
/// 3. Using probabilistic decision (score as probability) to continue or stop
/// 4. Limiting sequence length to avoid overly long monologues
fn generate_speech_sequence(corpus: &TrialCorpus, session: &mut TrialSession) -> Vec<usize> {
    let mut sequence = Vec::new();

    // Select initial question
    let first_speech = select_next_question(corpus, session);
    sequence.push(first_speech);

    // Generate continuation sequence using QQ ranks
    let mut current_speech = first_speech;

    while sequence.len() < session.max_continuation_depth as usize + 1 {
        if let Some(qq_rank) = corpus.qq_ranks.get(current_speech)
            && !qq_rank.is_empty()
        {
            // Filter available continuations (not yet asked)
            let available_ranks: Vec<_> = qq_rank
                .iter()
                .filter(|rank| !session.has_asked(rank.answer_index))
                .cloned()
                .collect();

            if !available_ranks.is_empty() {
                let best_score = available_ranks[0].score;

                // Use score as probability: higher similarity = more likely to continue
                if session.should_continue(best_score) {
                    // Sample a continuation from available QQ ranks
                    let selected = sample_weighted_indices(
                        &available_ranks,
                        session.temperature,
                        1,
                        &mut session.rng,
                    );

                    if let Some(&next_speech) = selected.first() {
                        session.mark_asked(next_speech);
                        sequence.push(next_speech);
                        current_speech = next_speech;
                        log::info!(
                            "QQ continuation in sequence: {} → {} (score: {:.3})",
                            current_speech,
                            next_speech,
                            best_score
                        );
                        continue;
                    }
                }
            }
        }

        // No more continuations
        break;
    }

    // Update session state
    if let Some(&last_speech) = sequence.last() {
        session.set_last_diner_speech(last_speech);
        session.set_current_question(last_speech); // For contextual response evaluation
    }

    log::info!(
        "Generated speech sequence of length {}: {:?}",
        sequence.len(),
        sequence
    );
    sequence
}

/// Calculate relevance penalty for response based on QA ranks
///
/// Checks if the player's response is relevant to the current question using QA ranks.
/// Returns 0.0 if relevant, negative penalty if irrelevant.
fn calculate_relevance_penalty(
    question_idx: usize,
    response_idx: usize,
    corpus: &TrialCorpus,
) -> f32 {
    // Get QA ranks for this question
    if let Some(question_ranks) = corpus.qa_ranks.get(question_idx) {
        // Check all keywords in this question
        for keyword_ranks in question_ranks {
            // Check if response is in the top ranks for any keyword
            if let Some(_rank) = keyword_ranks
                .iter()
                .find(|r| r.answer_index == response_idx)
            {
                // Found in ranks - check position
                let position = keyword_ranks
                    .iter()
                    .position(|r| r.answer_index == response_idx)
                    .unwrap();

                if position < 5 {
                    // Top 5 - highly relevant, no penalty
                    return 0.0;
                } else if position < 10 {
                    // Top 10 - somewhat relevant, small penalty
                    return -0.2;
                }
            }
        }

        // Response not found in top ranks for any keyword - irrelevant
        // Apply significant penalty
        return -0.5;
    }

    // No QA ranks available - can't evaluate relevance
    0.0
}

/// Select the next question to ask based on previous interactions
///
/// This creates a coherent dialogue flow by:
/// 1. Using AQ ranks (answer → question) to select questions related to player's last response
/// 2. Filtering out already-asked questions to avoid repetition
/// 3. Applying temperature-weighted sampling for variety while maintaining relevance
/// 4. Falling back to random selection if no history or ranks available
fn select_next_question(corpus: &TrialCorpus, session: &mut TrialSession) -> usize {
    // Reset continuation depth for new topic
    session.reset_continuation();

    // If there's a previous player response, use AQ ranks to select related question
    if let Some(last_response_idx) = session.last_response_index
        && let Some(aq_rank) = corpus.aq_ranks.get(last_response_idx)
    {
        // Filter available questions (not yet asked)
        let available_ranks: Vec<_> = aq_rank
            .iter()
            .filter(|rank| !session.has_asked(rank.answer_index))
            .cloned()
            .collect();

        if !available_ranks.is_empty() {
            // Use weighted sampling based on AQ ranks
            let selected =
                sample_weighted_indices(&available_ranks, session.temperature, 1, &mut session.rng);

            if let Some(&question_idx) = selected.first() {
                session.mark_asked(question_idx);
                return question_idx;
            }
        }
    }

    // Fallback: random available question
    let num_questions = corpus.diner_speeches.len();
    let available_questions = (0..num_questions)
        .filter(|&idx| !session.has_asked(idx))
        .collect::<Vec<_>>();

    // Pick a random available question.
    // If all questions asked, pick any random one
    let speech_index = available_questions
        .choose(&mut session.rng)
        .cloned()
        .unwrap_or_else(|| session.rng.random_range(0..num_questions));

    session.mark_asked(speech_index);

    speech_index
}

fn create_diner_statement_with_sequence(
    speech_indices: Vec<usize>,
    trial_registry: &TrialCorpus,
    trial_session: &mut TrialSession,
) -> TrialStatement {
    let temperature = trial_session.temperature;

    // Generate response options for EACH speech in the sequence
    // Player can interrupt and respond after any speech
    let mut options_sequence: Vec<Vec<Vec<TrialResponseOption>>> = Vec::new();

    for &speech_idx in &speech_indices {
        let speech = &trial_registry.diner_speeches[speech_idx];
        let mut speech_options: Vec<Vec<TrialResponseOption>> = Vec::new();
        let mut keyword_idx = 0;

        log::info!(
            "Creating response options for speech {} in sequence",
            speech_idx
        );

        for item in &speech.items {
            let TrialSpeechItem::Keyword(_) = item else {
                continue;
            };

            let options_count = trial_session.rng.random_range(2..=4);

            let selected = if let Some(question_ranks) = trial_registry.qa_ranks.get(speech_idx)
                && let Some(ranks) = question_ranks.get(keyword_idx)
            {
                log::debug!(
                    "> Sampling response options for speech {} keyword {} using ranks",
                    speech_idx,
                    keyword_idx
                );
                // Sample responses using temperature-weighted scores
                sample_weighted_indices(ranks, temperature, options_count, &mut trial_session.rng)
            } else {
                // Fallback: random sampling
                (0..trial_registry.responses.len())
                    .choose_multiple(&mut trial_session.rng, options_count)
            };

            speech_options.push(
                selected
                    .into_iter()
                    .map(|idx| {
                        let response = &trial_registry.responses[idx];
                        TrialResponseOption {
                            corpus_index: idx,
                            kind: response.kind.to_view(),
                            summary: response.summary.clone(),
                        }
                    })
                    .collect(),
            );

            keyword_idx += 1;
        }

        options_sequence.push(speech_options);
    }

    // Convert speech indices to view speeches
    let speech_sequence: Vec<_> = speech_indices
        .iter()
        .map(|&idx| trial_registry.diner_speeches[idx].to_view())
        .collect();

    log::info!(
        "Generated {} speeches with {} option sets",
        speech_sequence.len(),
        options_sequence.len()
    );

    TrialStatement {
        speech_sequence,
        options_sequence,
    }
}

/// Sample indices from ranks using temperature-weighted softmax
fn sample_weighted_indices(
    ranks: &[TrialQARank],
    temperature: f32,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<usize> {
    if ranks.is_empty() {
        return Vec::new();
    }

    // Apply temperature and compute softmax weights
    let temp = temperature.max(0.01); // Avoid division by zero
    let max_score = ranks
        .iter()
        .map(|r| r.score)
        .fold(f32::NEG_INFINITY, f32::max);

    let choices: Vec<_> = ranks
        .iter()
        .map(|r| {
            let scaled = (r.score - max_score) / temp; // Subtract max for numerical stability
            let weight = scaled.exp() as f64;
            (r.answer_index, weight)
        })
        .collect();

    choices
        .choose_multiple_weighted(rng, count, |item| item.1)
        .unwrap()
        .map(|item| item.0)
        .collect()
}
