//! The trial system. It works outside the ECS loop.

use bevy_ecs::system::SystemState;
use dishaster_models::{
    TrialCorpus, TrialIntro, TrialParticipantAppearance, TrialQARank, TrialResponseOption,
    TrialSpeech, TrialSpeechItem, TrialStatement,
};

use crate::{prelude::*, resources::*, sim::Simulation};

impl Simulation {
    pub(super) fn create_trial_intro(&mut self) -> TrialIntro {
        let mut rng = self.world.resource_mut::<GameRng>();
        TrialIntro {
            left: random_appearance(&mut rng),
            right: random_appearance(&mut rng),
        }
    }

    pub(super) fn create_diner_statement(&mut self) -> TrialStatement {
        let mut system_state: SystemState<(
            Res<GameModelRegistryRes>,
            ResMut<TrialSession>,
            ResMut<GameRng>,
        )> = SystemState::new(&mut self.world);
        let (registry, mut trial_session, mut rng) = system_state.get_mut(&mut self.world);

        let speech_index = select_next_question(&registry.trial, &mut trial_session, &mut rng);
        println!("Selected diner speech index: {}", speech_index);

        create_diner_statement_with_speech(speech_index, &registry.trial, &trial_session, &mut rng)
    }

    pub(super) fn trial_respond(&mut self, resp_corpus_index: usize) -> TrialSpeech {
        // Record the player's response choice
        let mut trial_session = self.world.resource_mut::<TrialSession>();
        trial_session.set_last_response(resp_corpus_index);

        // Respond with the selected speech
        let registry = self.world.resource::<GameModelRegistryRes>();
        registry.trial.responses[resp_corpus_index].content.clone()
    }
}

fn random_appearance(rng: &mut GameRng) -> TrialParticipantAppearance {
    TrialParticipantAppearance {
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
fn select_next_question(
    trial_registry: &TrialCorpus,
    trial_session: &mut TrialSession,
    rng: &mut GameRng,
) -> usize {
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
            let selected =
                sample_weighted_indices(&available_ranks, trial_session.temperature, 1, rng);

            if let Some(&question_idx) = selected.first() {
                return question_idx;
            }
        }
    }

    // Fallback: random available question
    let available_questions: Vec<usize> = (0..trial_registry.diner_speeches.len())
        .filter(|&idx| !trial_session.has_asked(idx))
        .collect();

    let speech_index = if available_questions.is_empty() {
        // All questions asked, pick any random one
        rng.random_range(0..trial_registry.diner_speeches.len())
    } else {
        // Pick a random available question
        *available_questions.choose(rng).unwrap()
    };
    trial_session.mark_asked(speech_index);

    speech_index
}

fn create_diner_statement_with_speech(
    speech_index: usize,
    trial_registry: &TrialCorpus,
    trial_session: &TrialSession,
    rng: &mut GameRng,
) -> TrialStatement {
    let speech = trial_registry.diner_speeches[speech_index].clone();
    let temperature = trial_session.temperature;

    let mut options: Vec<Vec<TrialResponseOption>> = Vec::new();
    let mut keyword_idx = 0;

    println!("Creating diner statement for speech: {speech:?}");

    for item in &speech.items {
        let TrialSpeechItem::Keyword(_) = item else {
            continue;
        };

        let options_count = rng.random_range(2..=4);
        let selected = if let Some(question_ranks) = trial_registry.qa_ranks.get(speech_index)
            && let Some(ranks) = question_ranks.get(keyword_idx)
        {
            println!(
                "> Sampling response options for keyword {} using ranks: {:?}",
                keyword_idx, ranks
            );
            // Sample responses using temperature-weighted scores
            sample_weighted_indices(ranks, temperature, options_count, rng)
        } else {
            // Fallback: random sampling
            (0..trial_registry.responses.len()).choose_multiple(rng, options_count)
        };

        options.push(
            selected
                .into_iter()
                .map(|idx| {
                    let response = &trial_registry.responses[idx];
                    TrialResponseOption {
                        corpus_index: idx,
                        kind: response.kind,
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
    rng: &mut GameRng,
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
