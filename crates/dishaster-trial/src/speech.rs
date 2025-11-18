use dishaster_models::{TrialCorpus, TrialRank};
use dishaster_views::{
    SpeechId, TrialIntro, TrialParticipantAppearance as TrialParticipantAppearanceView,
    TrialResponseOption, TrialSpeech, TrialStatement,
};

use crate::{
    PsychImpact, ReputationImpact, TrialConfig, TrialImpact, TrialSession, adapter::*, prelude::*,
};

/// Creates introductory trial information with random participant appearances.
///
/// Generates random emotion and gesture combinations for both the diner (left)
/// and the manager (right) to provide visual variety in the trial UI.
pub fn create_trial_intro(session: &mut TrialSession) -> TrialIntro {
    // Currently, emit random appearances for both sides.
    TrialIntro {
        left: random_appearance(&mut session.rng),
        right: random_appearance(&mut session.rng),
    }
}

fn random_appearance(rng: &mut impl Rng) -> TrialParticipantAppearanceView {
    const EMOTIONS: &[char] = &[
        '😅', '😡', '😠', '😤', '😞', '😢', '😭', '😰', '😨', '😱', '😠',
    ];
    const GESTURES: &[char] = &[
        '👍', '👎', '👊', '🤚', '✋', '👋', '🤞', '🤏', '👈', '👉', '🤝', '👍', '👏',
    ];

    TrialParticipantAppearanceView {
        emotion: EMOTIONS.choose(rng).copied().unwrap(),
        gesture: GESTURES.choose(rng).copied(),
    }
}

/// Generates a coherent diner statement using QQ-rank based speech sequencing.
///
/// Creates a multi-turn statement where each speech item is selected based on
/// semantic similarity (QQ ranks) to maintain topical coherence. Uses the session's
/// trigger topic to filter relevant speeches.
pub fn create_diner_statement(session: &mut TrialSession, corpus: &TrialCorpus) -> TrialStatement {
    // Generate a sequence of related speeches (topic-centered)
    let speech_sequence = generate_speech_sequence(session, corpus);

    log::info!("Generated speech sequence: {:?}", speech_sequence);

    create_diner_statement_with_sequence(speech_sequence, corpus)
}

/// Generate response candidates for a specific question keyword (lazy loading).
///
/// Called when player selects a keyword to respond to. Uses QA ranks to find
/// contextually relevant responses with temperature-weighted sampling.
///
/// Use cache to ensure consistent options for repeated requests.
pub fn generate_trial_response_candidates(
    session: &mut TrialSession,
    corpus: &TrialCorpus,
    speech_id: SpeechId,
    keyword_index: usize,
) -> Vec<TrialResponseOption> {
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
    let options = generate_response_options(session, corpus, speech_id, keyword_index);
    session.cached_options.insert(key, options.clone());
    options
}

/// Processes player's response selection and generates follow-up content.
///
/// Evaluates the response's contextual relevance using QA ranks, applies impacts
/// to reputation and psychology, and generates a follow-up manager statement using
/// RR-rank based response sequencing.
///
/// Returns both the manager's follow-up statement and the combined impact of the response.
pub fn trial_respond(
    session: &mut TrialSession,
    corpus: &TrialCorpus,
    resp_id: SpeechId,
) -> (TrialStatement, TrialImpact) {
    // Get current question index and response data, then build speech sequence
    let (current_question_id, mut response_score, speech_sequence) = {
        let current_question_id = session.current_question_id;
        let response = &corpus.responses[resp_id as usize];
        let response_score = response.response_score;

        // Build speech sequence: start with the main response, then follow RR ranks for continuations
        let speech_sequence = generate_response_sequence(session, corpus, resp_id);

        (current_question_id, response_score, speech_sequence)
    };

    // Record the player's response choice
    session.set_last_response(resp_id);

    // Contextual evaluation: check if response is relevant to the current question
    // Use QA ranks to measure relevance (higher score = more relevant)
    if let Some(question_id) = current_question_id {
        let relevance_penalty =
            calculate_relevance_penalty(corpus, question_id, resp_id, &session.config);

        if relevance_penalty < 0.0 {
            log::info!(
                "Response {} to question {} is irrelevant (penalty: {:.3})",
                resp_id,
                question_id,
                relevance_penalty
            );
            response_score += relevance_penalty; // Apply penalty
        }
    }

    // Apply impacts and emit event
    let impact = get_trial_impacts(response_score, false, &session.config);

    (TrialStatement { speech_sequence }, impact)
}

/// After player responds, check if diner should continue with new topic.
/// This uses AQ ranks to select questions related to the player's last response.
pub fn trial_should_continue(session: &mut TrialSession, corpus: &TrialCorpus) -> bool {
    // Check if there are available follow-up questions based on player's response
    if let Some(last_response_id) = session.last_response_id
        && let Some(aq_rank) = corpus.aq_ranks.get(last_response_id as usize)
    {
        // Filter available questions (not yet asked)
        let available_ranks: Vec<_> = aq_rank
            .iter()
            .filter(|rank| !session.has_asked(rank.answer_id))
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
        session
            .rng
            .random_bool(session.config.no_rank_continuation_prob as f64)
    }
}

/// Apply penalty when trial times out without player response
///
/// This affects both reputation and psychological state:
/// - Reputation: Negative impact (ignoring customer concerns)
/// - Psych state: Mood and trust penalties for the trial diner
pub fn get_trial_timeout_penalty(config: &TrialConfig) -> TrialImpact {
    get_trial_impacts(config.timeout_response_score, true, config)
}

/// Apply impacts from trial interactions to both reputation and diner psychology
///
/// This is the core feedback application system for trials, affecting:
/// - Global reputation based on response quality
/// - Diner's mood, trust, and patience based on how they were treated
///
/// Returns a view of the impacts for GUI display.
fn get_trial_impacts(response_score: f32, is_timeout: bool, config: &TrialConfig) -> TrialImpact {
    // Apply reputation impact
    let reputation_impact = ReputationImpact { response_score };

    // Apply psychological impact to the diner
    let psych_impact = get_psych_impact(response_score, is_timeout, config);

    TrialImpact {
        reputation: reputation_impact,
        psych: psych_impact,
    }
}

fn get_psych_impact(response_score: f32, is_timeout: bool, config: &TrialConfig) -> PsychImpact {
    // Calculate psychological impacts based on response quality
    // Good responses (positive score) improve mood/trust, bad ones decrease
    let mood_change = if is_timeout {
        config.timeout_mood_penalty
    } else {
        response_score * config.mood_scale
    };

    let trust_change = if is_timeout {
        config.timeout_trust_penalty
    } else {
        response_score * config.trust_scale
    };

    let patience_change = if is_timeout {
        config.timeout_patience_penalty
    } else {
        response_score * config.patience_scale
    };

    PsychImpact {
        mood_change,
        trust_change,
        patience_change,
    }
}

/// Generate a sequence of related speeches using QQ ranks for topic coherence
///
/// This creates multi-turn diner dialogue by:
/// 1. Selecting an initial question using AQ ranks or random selection
/// 2. Following QQ continuation chains based on semantic similarity
/// 3. Using probabilistic decision (score as probability) to continue or stop
/// 4. Limiting sequence length to avoid overly long monologues
fn generate_speech_sequence(session: &mut TrialSession, corpus: &TrialCorpus) -> Vec<SpeechId> {
    let mut sequence = Vec::new();

    // Select initial question
    let first_speech = select_next_question(session, corpus);
    sequence.push(first_speech);

    // Generate continuation sequence using QQ ranks
    let mut current_speech = first_speech;

    while sequence.len() <= session.max_continuation_depth as usize {
        let Some(qq_rank) = corpus.qq_ranks.get(current_speech as usize) else {
            break;
        };

        // Filter available continuations (not yet asked)
        let available_ranks: Vec<_> = qq_rank
            .iter()
            .filter(|rank| !session.has_asked(rank.answer_id))
            .cloned()
            .collect();

        if available_ranks.is_empty() {
            break;
        }

        let best_score = available_ranks[0].score * session.config.qq_continuation_multiplier;

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
    session: &mut TrialSession,
    corpus: &TrialCorpus,
    resp_id: SpeechId,
) -> Vec<TrialSpeech> {
    let mut sequence = Vec::new();
    let mut used_responses = FxHashSet::default();

    // Start with the main response
    let response = &corpus.responses[resp_id as usize];
    sequence.push(response.content.to_view_with_id(resp_id));
    used_responses.insert(resp_id);

    // Generate continuation sequence using RR ranks
    let mut current_response = resp_id;

    while sequence.len() <= session.max_continuation_depth as usize {
        let Some(rr_rank) = corpus.rr_ranks.get(current_response as usize) else {
            break;
        };

        // Filter available continuations (exclude already used responses to avoid repetition)
        let available_ranks: Vec<_> = rr_rank
            .iter()
            .filter(|rank| !used_responses.contains(&rank.answer_id))
            .cloned()
            .collect();

        if available_ranks.is_empty() {
            break;
        }

        let best_score = available_ranks[0].score * session.config.rr_continuation_multiplier;

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

        let next_response_content = &corpus.responses[next_response as usize].content;
        sequence.push(next_response_content.to_view_with_id(next_response));
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
        sequence.iter().map(|s| s.id).collect::<Vec<_>>()
    );
    sequence
}

/// Calculate relevance penalty for response based on QA ranks
///
/// Checks if the player's response is relevant to the current question using QA ranks.
/// Returns 0.0 if relevant, negative penalty if irrelevant.
/// Uses score-based thresholds rather than position for more accurate relevance assessment.
fn calculate_relevance_penalty(
    corpus: &TrialCorpus,
    question_id: SpeechId,
    response_id: SpeechId,
    config: &TrialConfig,
) -> f32 {
    // Get QA ranks for this question
    let Some(question_ranks) = corpus.qa_ranks.get(question_id as usize) else {
        return 0.0;
    };

    // Find the best score for this response across all keywords
    let mut best_score: Option<f32> = None;

    for keyword_ranks in question_ranks {
        if let Some(rank) = keyword_ranks.iter().find(|r| r.answer_id == response_id) {
            best_score = Some(best_score.map_or(rank.score, |s| s.max(rank.score)));
        }
    }

    // Apply penalty based on best relevance score
    match best_score {
        Some(score) if score >= config.relevance_high_threshold => {
            // Highly relevant - no penalty
            0.0
        }
        Some(score) if score >= config.relevance_medium_threshold => {
            // Somewhat relevant - small penalty
            config.relevance_medium_penalty
        }
        Some(_) => {
            // Low relevance score - significant penalty
            config.relevance_low_penalty
        }
        None => {
            // Response not found in ranks for any keyword - irrelevant
            config.relevance_low_penalty
        }
    }
}

/// Select the next question to ask based on previous interactions
///
/// This creates a coherent dialogue flow by:
/// 1. Using AQ ranks (answer → question) to select questions related to player's last response
/// 2. Filtering out already-asked questions to avoid repetition
/// 3. Filtering by topic if the trial was triggered by specific feedback
/// 4. Applying temperature-weighted sampling for variety while maintaining relevance
/// 5. Falling back to random selection if no history or ranks available
fn select_next_question(session: &mut TrialSession, corpus: &TrialCorpus) -> SpeechId {
    // Reset continuation depth for new topic
    session.reset_continuation();

    // Helper to check if a speech matches the trigger topic
    let matches_topic = |speech_id: SpeechId| -> bool {
        // Speech must either have matching topic or no topic (general)
        // General speeches can be used for any topic
        session.trigger_topic.is_none_or(|trigger_topic| {
            corpus
                .diner_speeches
                .get(speech_id as usize)
                .is_some_and(|speech| speech.topic.is_none_or(|topic| topic == trigger_topic))
        })
    };

    // If there's a previous player response, use AQ ranks to select related question
    if let Some(last_response_id) = session.last_response_id
        && let Some(aq_rank) = corpus.aq_ranks.get(last_response_id as usize)
    {
        // Filter available questions (not yet asked AND matching topic)
        let available_ranks: Vec<_> = aq_rank
            .iter()
            .filter(|rank| !session.has_asked(rank.answer_id) && matches_topic(rank.answer_id))
            .cloned()
            .collect();

        if !available_ranks.is_empty() {
            // Use weighted sampling based on AQ ranks
            let selected =
                sample_weighted_indices(&available_ranks, session.temperature, 1, &mut session.rng);

            if let Some(&question_id) = selected.first() {
                session.mark_asked(question_id);
                return question_id;
            }
        }
    }

    // Fallback: random available question (filtered by topic)
    let num_questions = corpus.diner_speeches.len() as SpeechId;
    let available_questions: Vec<_> = (0..num_questions)
        .filter(|&id| !session.has_asked(id) && matches_topic(id))
        .collect();

    // Pick a random available question.
    // If no topic-matching questions available, allow any available question
    let speech_index = available_questions
        .choose(&mut session.rng)
        .cloned()
        .unwrap_or_else(|| {
            // No topic-matching questions - fall back to any available or any random
            let all_available: Vec<_> = (0..num_questions)
                .filter(|&id| !session.has_asked(id))
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
    speech_ids: Vec<SpeechId>,
    corpus: &TrialCorpus,
) -> TrialStatement {
    // Convert speech indices to view speeches
    let speech_sequence: Vec<_> = speech_ids
        .into_iter()
        .map(|id| corpus.diner_speeches[id as usize].to_view_with_id(id))
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
    session: &mut TrialSession,
    corpus: &TrialCorpus,
    speech_id: SpeechId,
    keyword_idx: usize,
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

    let selected = if let Some(question_ranks) = corpus.qa_ranks.get(speech_id as usize)
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
        (0..corpus.responses.len() as SpeechId).choose_multiple(&mut session.rng, options_count)
    };

    selected
        .into_iter()
        .map(|id| {
            let response = &corpus.responses[id as usize];
            TrialResponseOption {
                id,
                kind: response.kind.to_view(),
                summary: response.summary.clone(),
            }
        })
        .collect()
}

/// Sample indices from ranks using temperature-weighted softmax
fn sample_weighted_indices(
    ranks: &[TrialRank],
    temperature: f32,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<SpeechId> {
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
            (r.answer_id, weight)
        })
        .collect();

    choices
        .choose_multiple_weighted(rng, count, |item| item.1)
        .unwrap()
        .map(|item| item.0)
        .collect()
}
