//! The trial system. It works outside the ECS loop.

use bevy_ecs::system::SystemState;
use dishaster_interface::SimEvent;

use crate::{
    components::DinerPsychState,
    models::{TrialCorpus, TrialQARank},
    prelude::*,
    resources::*,
    sim::Simulation,
    views::{self, *},
};

impl Simulation {
    pub(super) fn create_trial_intro(&mut self) -> TrialIntro {
        let mut session = self.world.resource_mut::<TrialSession>();
        TrialIntro {
            left: random_appearance(&mut session.rng),
            right: random_appearance(&mut session.rng),
        }
    }

    pub(super) fn create_diner_statement(&mut self) -> TrialStatement {
        let mut system_state: SystemState<(Res<GameModelRegistryRes>, ResMut<TrialSession>)> =
            SystemState::new(&mut self.world);
        let (registry, mut session) = system_state.get_mut(&mut self.world);

        // Generate a sequence of related speeches (topic-centered)
        let speech_sequence = generate_speech_sequence(&registry.trial, &mut session);

        log::info!("Generated speech sequence: {:?}", speech_sequence);

        create_diner_statement_with_sequence(speech_sequence, &registry.trial, &mut session)
    }

    /// Generate response candidates for a specific question keyword (lazy loading).
    ///
    /// Called when player selects a keyword to respond to. Uses QA ranks to find
    /// contextually relevant responses with temperature-weighted sampling.
    ///
    /// Use cache to ensure consistent options for repeated requests.
    pub(super) fn generate_trial_response_candidates(
        &mut self,
        speech_id: usize,
        keyword_index: usize,
    ) -> Vec<TrialResponseOption> {
        let mut system_state: SystemState<(Res<GameModelRegistryRes>, ResMut<TrialSession>)> =
            SystemState::new(&mut self.world);
        let (registry, mut session) = system_state.get_mut(&mut self.world);

        // Check cache first
        let key = (speech_id, keyword_index);
        if let Some(cached) = session.cached_options.get(&key) {
            log::info!(
                "Using cached response options for speech {} keyword {}",
                speech_id,
                keyword_index
            );
            return cached.clone();
        }
        let options =
            generate_response_options(speech_id, keyword_index, &registry.trial, &mut session);
        session.cached_options.insert(key, options.clone());
        options
    }

    pub(super) fn trial_respond(&mut self, resp_id: usize) -> views::TrialStatement {
        // Get current question index and response data, then build speech sequence
        let (current_question_idx, mut response_score, speech_sequence) = {
            let mut system_state: SystemState<(Res<GameModelRegistryRes>, ResMut<TrialSession>)> =
                SystemState::new(&mut self.world);
            let (registry, mut session) = system_state.get_mut(&mut self.world);

            let current_question_idx = session.current_question_index;
            let response = &registry.trial.responses[resp_id];
            let response_score = response.response_score;

            // Build speech sequence: start with the main response, then follow RR ranks for continuations
            let speech_sequence =
                generate_response_sequence(resp_id, &registry.trial, &mut session);

            (current_question_idx, response_score, speech_sequence)
        };

        // Record the player's response choice
        let mut session = self.world.resource_mut::<TrialSession>();
        session.set_last_response(resp_id);

        // Contextual evaluation: check if response is relevant to the current question
        // Use QA ranks to measure relevance (higher rank = more relevant)
        if let Some(question_idx) = current_question_idx {
            let registry = self.world.resource::<GameModelRegistryRes>();
            let relevance_penalty =
                calculate_relevance_penalty(question_idx, resp_id, &registry.trial);

            if relevance_penalty < 0.0 {
                log::info!(
                    "Response {} to question {} is irrelevant (penalty: {:.3})",
                    resp_id,
                    question_idx,
                    relevance_penalty
                );
                response_score += relevance_penalty; // Apply penalty
            }
        }

        // Apply impacts and emit event
        let impact = self.apply_trial_impacts(response_score, false);
        let mut events = self.world.resource_mut::<EventQueue>();
        events.push(SimEvent::TrialImpact(impact.into()));

        views::TrialStatement { speech_sequence }
    }

    /// After player responds, check if diner should continue with new topic.
    /// This uses AQ ranks to select questions related to the player's last response.
    pub(super) fn trial_should_continue(&mut self) -> bool {
        let mut system_state: SystemState<(Res<GameModelRegistryRes>, ResMut<TrialSession>)> =
            SystemState::new(&mut self.world);
        let (registry, mut session) = system_state.get_mut(&mut self.world);

        // Check if there are available follow-up questions based on player's response
        if let Some(last_response_idx) = session.last_response_id
            && let Some(aq_rank) = registry.trial.aq_ranks.get(last_response_idx)
        {
            // Filter available questions (not yet asked)
            let available_ranks: Vec<_> = aq_rank
                .iter()
                .filter(|rank| !session.has_asked(rank.answer_index))
                .cloned()
                .collect();

            if available_ranks.is_empty() {
                return false;
            }
            // Use best score as probability to continue
            let best_score = available_ranks[0].score;
            session.should_continue(best_score)
        } else {
            // No response history or ranks - randomly decide
            session.rng.random_bool(0.3)
        }
    }

    /// Apply penalty when trial times out without player response
    ///
    /// This affects both reputation and psychological state:
    /// - Reputation: Negative impact (ignoring customer concerns)
    /// - Psych state: Mood and trust penalties for the trial diner
    pub(super) fn apply_trial_timeout_penalty(&mut self) {
        // Use a more severe response score for timeout
        let timeout_response_score = -0.8; // Worse than a poor response
        let impact = self.apply_trial_impacts(timeout_response_score, true);

        // Emit impact event for GUI
        let mut events = self.world.resource_mut::<EventQueue>();
        events.push(SimEvent::TrialImpact(impact.into()));

        log::info!("Applied trial timeout penalty");
    }

    /// Apply impacts from trial interactions to both reputation and diner psychology
    ///
    /// This is the core feedback application system for trials, affecting:
    /// - Global reputation based on response quality
    /// - Diner's mood, trust, and patience based on how they were treated
    ///
    /// Returns a view of the impacts for GUI display.
    fn apply_trial_impacts(&mut self, response_score: f32, is_timeout: bool) -> TrialImpactView {
        // Apply reputation impact
        let reputation_impact = self.apply_reputation_impact(response_score);

        // Apply psychological impact to the diner
        let psych_impact = self.apply_psych_impact(response_score, is_timeout);

        TrialImpactView {
            psych_impact,
            reputation_impact,
        }
    }

    fn apply_reputation_impact(&mut self, response_score: f32) -> Option<ReputationView> {
        let mut system_state: SystemState<(ResMut<ReputationStateRes>, Res<ReputationConfigRes>)> =
            SystemState::new(&mut self.world);
        let (mut reputation, reputation_config) = system_state.get_mut(&mut self.world);

        let base_impact = reputation_config.base_impacts.quality;
        let old_reputation = reputation.reputation;

        reputation.apply_feedback_impact(base_impact, response_score, &reputation_config);

        let reputation_delta = reputation.reputation - old_reputation;

        log::info!(
            "Trial response impact on reputation: {:.2} (score: {:.2})",
            reputation_delta,
            response_score
        );

        Some(ReputationView {
            reputation: reputation.reputation,
            reputation_delta,
            fsri: reputation.fsri,
            food_quality: reputation.food_quality,
        })
    }

    fn apply_psych_impact(
        &mut self,
        response_score: f32,
        is_timeout: bool,
    ) -> Option<PsychImpactView> {
        let session = self.world.resource::<TrialSession>();
        let diner_entity = session.diner_entity?;

        let mut system_state: SystemState<Query<&mut DinerPsychState>> =
            SystemState::new(&mut self.world);
        let mut diner_query = system_state.get_mut(&mut self.world);

        let mut psych_state = diner_query.get_mut(diner_entity).ok()?;

        let old_mood = psych_state.mood;
        let old_trust = psych_state.trust;
        let old_patience = psych_state.patience;

        // Calculate psychological impacts based on response quality
        // Good responses (positive score) improve mood/trust, bad ones decrease
        let mood_change = if is_timeout {
            -0.15 // Significant mood penalty for being ignored
        } else {
            response_score * 0.1 // Scale response score to mood change
        };

        let trust_change = if is_timeout {
            -0.1 // Trust penalty for being ignored
        } else {
            response_score * 0.05 // Smaller trust impact
        };

        let patience_change = if is_timeout {
            -5.0 // Reduce patience significantly for timeout
        } else {
            response_score * 2.0 // Patience affected by response quality
        };

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

        Some(PsychImpactView {
            mood_delta,
            trust_delta,
            patience_delta,
        })
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

    while sequence.len() <= session.max_continuation_depth as usize {
        let Some(qq_rank) = corpus.qq_ranks.get(current_speech) else {
            break;
        };

        // Filter available continuations (not yet asked)
        let available_ranks: Vec<_> = qq_rank
            .iter()
            .filter(|rank| !session.has_asked(rank.answer_index))
            .cloned()
            .collect();

        if available_ranks.is_empty() {
            break;
        }

        let best_score = available_ranks[0].score;

        // Use score as probability: higher similarity = more likely to continue
        if !session.should_continue(best_score) {
            break;
        }

        // Sample a continuation from available QQ ranks
        let selected =
            sample_weighted_indices(&available_ranks, session.temperature, 1, &mut session.rng);

        let Some(&next_speech) = selected.first() else {
            break;
        };

        session.mark_asked(next_speech);
        sequence.push(next_speech);
        current_speech = next_speech;
        log::info!(
            "QQ continuation in sequence: {} → {} (score: {:.3})",
            current_speech,
            next_speech,
            best_score
        );
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

/// Generate a sequence of related player responses using RR ranks for topic coherence
///
/// This creates multi-statement player responses by:
/// 1. Starting with the selected response
/// 2. Following RR continuation chains based on semantic similarity
/// 3. Using probabilistic decision (score as probability) to continue or stop
/// 4. Limiting sequence length to avoid overly long responses
fn generate_response_sequence(
    resp_id: usize,
    corpus: &TrialCorpus,
    session: &mut TrialSession,
) -> Vec<views::TrialSpeech> {
    let mut sequence = Vec::new();
    let mut used_responses = FxHashSet::default();

    // Start with the main response
    let response = &corpus.responses[resp_id];
    sequence.push(response.content.to_view_with_index(resp_id));
    used_responses.insert(resp_id);

    // Generate continuation sequence using RR ranks
    let mut current_response = resp_id;

    while sequence.len() <= session.max_continuation_depth as usize {
        let Some(rr_rank) = corpus.rr_ranks.get(current_response) else {
            break;
        };

        // Filter available continuations (exclude already used responses to avoid repetition)
        let available_ranks: Vec<_> = rr_rank
            .iter()
            .filter(|rank| !used_responses.contains(&rank.answer_index))
            .cloned()
            .collect();

        if available_ranks.is_empty() {
            break;
        }

        let best_score = available_ranks[0].score * 0.2;

        // Use score as probability: higher similarity = more likely to continue
        if !session.should_continue(best_score) {
            break;
        }

        // Sample a continuation from available RR ranks
        let selected =
            sample_weighted_indices(&available_ranks, session.temperature, 1, &mut session.rng);

        let Some(&next_response) = selected.first() else {
            break;
        };

        let next_response_content = &corpus.responses[next_response].content;
        sequence.push(next_response_content.to_view_with_index(next_response));
        used_responses.insert(next_response);
        current_response = next_response;

        log::info!(
            "RR continuation in response sequence: {} → {} (score: {:.3})",
            resp_id,
            next_response,
            best_score
        );
    }

    log::info!(
        "Generated response sequence of length {}: {:?}",
        sequence.len(),
        sequence.iter().map(|s| s.index).collect::<Vec<_>>()
    );
    sequence
}

/// Calculate relevance penalty for response based on QA ranks
///
/// Checks if the player's response is relevant to the current question using QA ranks.
/// Returns 0.0 if relevant, negative penalty if irrelevant.
fn calculate_relevance_penalty(
    question_id: usize,
    response_id: usize,
    corpus: &TrialCorpus,
) -> f32 {
    // Get QA ranks for this question
    let Some(question_ranks) = corpus.qa_ranks.get(question_id) else {
        return 0.0;
    };

    // Check all keywords in this question
    for keyword_ranks in question_ranks {
        // Check if response is in the top ranks for any keyword
        if keyword_ranks.iter().any(|r| r.answer_index == response_id) {
            // Found in ranks - check position
            let position = keyword_ranks
                .iter()
                .position(|r| r.answer_index == response_id)
                .unwrap();

            // FIXME: should use score instead of position
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
    -0.5
}

/// Select the next question to ask based on previous interactions
///
/// This creates a coherent dialogue flow by:
/// 1. Using AQ ranks (answer → question) to select questions related to player's last response
/// 2. Filtering out already-asked questions to avoid repetition
/// 3. Filtering by topic if the trial was triggered by specific feedback
/// 4. Applying temperature-weighted sampling for variety while maintaining relevance
/// 5. Falling back to random selection if no history or ranks available
fn select_next_question(corpus: &TrialCorpus, session: &mut TrialSession) -> usize {
    // Reset continuation depth for new topic
    session.reset_continuation();

    // Helper to check if a speech matches the trigger topic
    let matches_topic = |speech_id: usize| -> bool {
        // Speech must either have matching topic or no topic (general)
        // General speeches can be used for any topic
        session.trigger_topic.is_none_or(|trigger_topic| {
            corpus
                .diner_speeches
                .get(speech_id)
                .is_some_and(|speech| speech.topic.is_none_or(|topic| topic == trigger_topic))
        })
    };

    // If there's a previous player response, use AQ ranks to select related question
    if let Some(last_response_id) = session.last_response_id
        && let Some(aq_rank) = corpus.aq_ranks.get(last_response_id)
    {
        // Filter available questions (not yet asked AND matching topic)
        let available_ranks: Vec<_> = aq_rank
            .iter()
            .filter(|rank| {
                !session.has_asked(rank.answer_index) && matches_topic(rank.answer_index)
            })
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

    // Fallback: random available question (filtered by topic)
    let num_questions = corpus.diner_speeches.len();
    let available_questions: Vec<_> = (0..num_questions)
        .filter(|&idx| !session.has_asked(idx) && matches_topic(idx))
        .collect();

    // Pick a random available question.
    // If no topic-matching questions available, allow any available question
    let speech_index = available_questions
        .choose(&mut session.rng)
        .cloned()
        .unwrap_or_else(|| {
            // No topic-matching questions - fall back to any available or any random
            let all_available: Vec<_> = (0..num_questions)
                .filter(|&idx| !session.has_asked(idx))
                .collect();

            all_available
                .choose(&mut session.rng)
                .cloned()
                .unwrap_or_else(|| session.rng.random_range(0..num_questions))
        });

    session.mark_asked(speech_index);

    speech_index
}

fn create_diner_statement_with_sequence(
    speech_ids: Vec<usize>,
    corpus: &TrialCorpus,
    _session: &mut TrialSession,
) -> TrialStatement {
    // Convert speech indices to view speeches
    let speech_sequence: Vec<_> = speech_ids
        .iter()
        .map(|&idx| corpus.diner_speeches[idx].to_view_with_index(idx))
        .collect();

    log::info!(
        "Generated speech sequence with {} speeches (lazy option generation)",
        speech_sequence.len()
    );

    TrialStatement { speech_sequence }
}

/// Generate response options for a specific question keyword.
///
/// Used for lazy loading - called only when player actually selects a keyword.
/// Uses QA ranks to find contextually relevant responses with temperature-weighted sampling.
fn generate_response_options(
    speech_id: usize,
    keyword_idx: usize,
    corpus: &TrialCorpus,
    session: &mut TrialSession,
) -> Vec<TrialResponseOption> {
    const WEIGHTED_OPTION_COUNTS: &[(usize, i32)] = &[(1, 1), (2, 3), (3, 4), (4, 2)];

    let temperature = session.temperature;
    let options_count = WEIGHTED_OPTION_COUNTS
        .choose_weighted(&mut session.rng, |it| it.1)
        .unwrap()
        .0;

    log::info!(
        "Generating {} response options for speech {} keyword {}",
        options_count,
        speech_id,
        keyword_idx
    );

    let selected = if let Some(question_ranks) = corpus.qa_ranks.get(speech_id)
        && let Some(ranks) = question_ranks.get(keyword_idx)
    {
        log::debug!("> Using QA ranks with {} candidates", ranks.len());
        // Sample responses using temperature-weighted scores
        sample_weighted_indices(ranks, temperature, options_count, &mut session.rng)
    } else {
        // Fallback: random sampling
        log::warn!(
            "No QA ranks found for speech {} keyword {}, using random selection",
            speech_id,
            keyword_idx
        );
        (0..corpus.responses.len()).choose_multiple(&mut session.rng, options_count)
    };

    selected
        .into_iter()
        .map(|idx| {
            let response = &corpus.responses[idx];
            TrialResponseOption {
                corpus_index: idx,
                kind: response.kind.to_view(),
                summary: response.summary.clone(),
            }
        })
        .collect()
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
