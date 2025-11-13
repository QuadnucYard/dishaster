//! The trial system. It works outside the ECS loop.

use bevy_ecs::system::SystemState;

use crate::{
    models::{TrialCorpus, TrialQARank},
    prelude::*,
    resources::*,
    sim::Simulation,
    views::{self, TrialIntro, TrialResponseOption, TrialSpeechItem, TrialStatement},
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

        let speech_index = select_next_question(&registry.trial, &mut trial_session);
        println!("Selected diner speech index: {}", speech_index);

        create_diner_statement_with_speech(speech_index, &registry.trial, &mut trial_session)
    }

    pub(super) fn trial_respond(&mut self, resp_corpus_index: usize) -> views::TrialSpeech {
        // Record the player's response choice
        let mut trial_session = self.world.resource_mut::<TrialSession>();
        trial_session.set_last_response(resp_corpus_index);

        // Get response and extract needed values
        let registry = self.world.resource::<GameModelRegistryRes>();
        let response_score = registry.trial.responses[resp_corpus_index].response_score;
        let speech = registry.trial.responses[resp_corpus_index]
            .content
            .to_view();

        // Apply reputation impact
        let mut system_state: SystemState<(ResMut<ReputationStateRes>, Res<ReputationConfigRes>)> =
            SystemState::new(&mut self.world);
        let (mut reputation, reputation_config) = system_state.get_mut(&mut self.world);

        // Apply a moderate base impact modified by response score
        // Using quality topic (-4.0) as a representative negative feedback
        let base_impact = reputation_config.base_impacts.quality;
        reputation.apply_feedback_impact(base_impact, response_score, &reputation_config);

        speech
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

/// Select the next question to ask based on previous response (if any) using AQ ranks
///
/// This creates a coherent dialogue flow by:
/// 1. Using AQ ranks (answer → question) to select questions related to the player's last response
/// 2. Filtering out already-asked questions to avoid repetition
/// 3. Applying temperature-weighted sampling for variety while maintaining relevance
/// 4. Falling back to random selection if no response history or ranks available
fn select_next_question(trial_registry: &TrialCorpus, trial_session: &mut TrialSession) -> usize {
    // If there's a previous response, use AQ ranks to select related question
    if let Some(last_response_idx) = trial_session.last_response_index
        && let Some(aq_rank) = trial_registry.aq_ranks.get(last_response_idx)
    {
        // Filter available questions (not yet asked)
        let available_ranks: Vec<_> = aq_rank
            .iter()
            .filter(|rank| !trial_session.has_asked(rank.answer_index))
            .cloned()
            .collect();

        if !available_ranks.is_empty() {
            // Use weighted sampling based on AQ ranks
            let selected = sample_weighted_indices(
                &available_ranks,
                trial_session.temperature,
                1,
                &mut trial_session.rng,
            );

            if let Some(&question_idx) = selected.first() {
                return question_idx;
            }
        }
    }

    // Fallback: random available question
    let num_questions = trial_registry.diner_speeches.len();
    let available_questions = (0..num_questions)
        .filter(|&idx| !trial_session.has_asked(idx))
        .collect::<Vec<_>>();

    // Pick a random available question.
    // If all questions asked, pick any random one
    let speech_index = available_questions
        .choose(&mut trial_session.rng)
        .cloned()
        .unwrap_or_else(|| trial_session.rng.random_range(0..num_questions));

    trial_session.mark_asked(speech_index);

    speech_index
}

fn create_diner_statement_with_speech(
    speech_index: usize,
    trial_registry: &TrialCorpus,
    trial_session: &mut TrialSession,
) -> TrialStatement {
    let speech = trial_registry.diner_speeches[speech_index].to_view();
    let temperature = trial_session.temperature;

    let mut options: Vec<Vec<TrialResponseOption>> = Vec::new();
    let mut keyword_idx = 0;

    println!("Creating diner statement for speech: {speech:?}");

    for item in &speech.items {
        let TrialSpeechItem::Keyword(_) = item else {
            continue;
        };

        let options_count = trial_session.rng.random_range(2..=4);
        let selected = if let Some(question_ranks) = trial_registry.qa_ranks.get(speech_index)
            && let Some(ranks) = question_ranks.get(keyword_idx)
        {
            println!(
                "> Sampling response options for keyword {} using ranks: {:?}",
                keyword_idx, ranks
            );
            // Sample responses using temperature-weighted scores
            sample_weighted_indices(ranks, temperature, options_count, &mut trial_session.rng)
        } else {
            // Fallback: random sampling
            (0..trial_registry.responses.len())
                .choose_multiple(&mut trial_session.rng, options_count)
        };

        options.push(
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

    println!(
        "> Generated response options: \n{}",
        options
            .iter()
            .map(|options| options
                .iter()
                .map(|option| format!(
                    "    [{:?}] {}: {}",
                    option.kind,
                    option.summary,
                    trial_registry.responses[option.corpus_index].content.text
                ))
                .collect::<Vec<_>>()
                .join("\n"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    TrialStatement { speech, options }
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
