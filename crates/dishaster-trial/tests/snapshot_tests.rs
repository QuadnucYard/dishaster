//! Snapshot tests for trial sessions using insta.
//!
//! Each test simulates a complete trial session and captures:
//! - All speeches generated
//! - Response options presented
//! - Player selections
//! - Impacts calculated
//! - Continuation decisions

use dishaster_models::{FeedbackTopic, TrialCorpus, TrialResponseKind};
use dishaster_trial::{
    TrialSession, create_diner_statement, create_trial_intro, generate_trial_response_candidates,
    trial_respond, trial_should_continue,
};
use dishaster_views::SpeechId;
use dishrupt_core::{EntityId, prelude::EcoString};
use serde::Serialize;

// === Load Real Corpus ===

/// Load the actual game corpus from data files
fn load_corpus() -> &'static TrialCorpus {
    use std::sync::{Arc, OnceLock};

    use dishaster_data::GameDataAssets;

    static DATA: OnceLock<Arc<GameDataAssets>> = OnceLock::new();

    let data = DATA.get_or_init(|| {
        let mut loader = dishaster_data::DataLoader::new("../../assets/data")
            .expect("Failed to create data loader");
        Arc::new(loader.load_all_data().expect("Failed to load game data"))
    });

    &data.models.trial
}

// === Test Utilities ===

/// Capture trial state for snapshot
#[derive(Debug, Serialize)]
struct TrialSnapshot {
    session_info: SessionInfo,
    turns: Vec<TurnSnapshot>,
    final_state: FinalState,
}

#[derive(Debug, Serialize)]
struct SessionInfo {
    seed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    trigger_topic: Option<FeedbackTopic>,
    max_continuation_depth: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct TurnSnapshot {
    turn_number: usize,
    diner_speeches: Vec<SpeechSnapshot>,
    response_options: Option<Vec<ResponseOptionsForSpeech>>,
    selected_response: Option<ResponseSelection>,
    manager_speeches: Option<Vec<SpeechSnapshot>>,
    impact: Option<ImpactSnapshot>,
    continuation_decision: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ResponseOptionsForSpeech {
    speech_id: SpeechId,
    keywords: Vec<ResponseOptionsForKeyword>,
}

#[derive(Debug, Serialize)]
struct ResponseOptionsForKeyword {
    keyword: EcoString,
    options: Vec<ResponseOptionSnapshot>,
}

#[derive(Debug, Serialize)]
struct ResponseSelection {
    speech_id: SpeechId,
    keyword_idx: usize,
    option_idx: SpeechId,
}

#[derive(Debug, Serialize)]
struct SpeechSnapshot {
    id: SpeechId,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<EcoString>,
    text: EcoString,
    emotion: char,
    #[serde(skip_serializing_if = "Option::is_none")]
    gesture: Option<char>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<FeedbackTopic>,
}

#[derive(Debug, Serialize)]
struct ResponseOptionSnapshot {
    id: SpeechId,
    kind: TrialResponseKind,
    summary: EcoString,
}

#[derive(Debug, Serialize)]
struct ImpactSnapshot {
    reputation_score: f32,
    mood_change: f32,
    trust_change: f32,
    patience_change: f32,
}

#[derive(Debug, Serialize)]
struct FinalState {
    total_turns: usize,
    asked_questions: Vec<SpeechId>,
    continuation_depth: u32,
}

// === Snapshot Tests ===

/// Run a random trial session and capture the complete flow
fn run_trial_snapshot(seed: u64, topic: Option<FeedbackTopic>) -> TrialSnapshot {
    use rand::prelude::*;

    let corpus = load_corpus();
    let mut session = TrialSession::new(seed);
    let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(seed.wrapping_mul(7919));
    let target_entity = EntityId::new(seed % 10 + 1).unwrap();

    session.start(target_entity, topic);

    let mut snapshot = TrialSnapshot {
        session_info: SessionInfo {
            seed,
            trigger_topic: topic,
            max_continuation_depth: session.max_continuation_depth,
            temperature: session.temperature,
        },
        turns: Vec::new(),
        final_state: FinalState {
            total_turns: 0,
            asked_questions: Vec::new(),
            continuation_depth: 0,
        },
    };

    let _intro = create_trial_intro(&mut session);

    // Run trial until it ends (max 10 turns to prevent infinite loops)
    for turn_num in 1..=10 {
        let statement = create_diner_statement(&mut session, corpus);
        let diner_speeches: Vec<_> = statement
            .speech_sequence
            .iter()
            .map(|s| SpeechSnapshot {
                id: s.id,
                summary: None,
                text: s.text.clone(),
                emotion: s.appearance.emotion,
                gesture: s.appearance.gesture,
                topic: corpus.diner_speeches[s.id as usize].topic,
            })
            .collect();

        // Randomly decide whether to respond or skip
        let should_respond = rng.random_bool(0.8); // 70% chance to respond

        if !should_respond || diner_speeches.is_empty() {
            // Player chose not to respond
            snapshot.turns.push(TurnSnapshot {
                turn_number: turn_num,
                diner_speeches,
                response_options: None,
                selected_response: None,
                manager_speeches: None,
                impact: None,
                continuation_decision: None,
            });
            break;
        }

        // Collect response options grouped by (speech_id, keyword_idx)
        let mut all_speech_options = Vec::new();
        let mut flat_options = Vec::new(); // For random selection

        for speech in &diner_speeches {
            let speech_data = &corpus.diner_speeches[speech.id as usize];

            // Count keywords in this speech
            let keywords = speech_data
                .items
                .iter()
                .filter_map(|item| {
                    if let dishaster_models::TrialSpeechItem::Keyword(kw) = item {
                        Some(kw.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let mut keywords_options = Vec::new();

            for (keyword_idx, keyword) in keywords.iter().enumerate() {
                let options = generate_trial_response_candidates(
                    &mut session,
                    corpus,
                    speech.id,
                    keyword_idx,
                );

                if !options.is_empty() {
                    let option_snapshots: Vec<_> = options
                        .into_iter()
                        .map(|o| {
                            flat_options.push((speech.id, keyword_idx, o.id));
                            ResponseOptionSnapshot {
                                id: o.id,
                                kind: unsafe {
                                    std::mem::transmute::<
                                        dishaster_views::TrialResponseKind,
                                        dishaster_models::TrialResponseKind,
                                    >(o.kind)
                                },
                                summary: o.summary.clone(),
                            }
                        })
                        .collect();

                    keywords_options.push(ResponseOptionsForKeyword {
                        keyword: keyword.clone(),
                        options: option_snapshots,
                    });
                }
            }

            if !keywords_options.is_empty() {
                all_speech_options.push(ResponseOptionsForSpeech {
                    speech_id: speech.id,
                    keywords: keywords_options,
                });
            }
        }

        if flat_options.is_empty() {
            // No valid responses, just record speeches
            snapshot.turns.push(TurnSnapshot {
                turn_number: turn_num,
                diner_speeches,
                response_options: None,
                selected_response: None,
                manager_speeches: None,
                impact: None,
                continuation_decision: None,
            });
            break;
        }

        // Randomly select a response from all available options
        let selected_idx = rng.random_range(0..flat_options.len());
        let (selected_speech_id, selected_keyword_idx, selected_option_id) =
            flat_options[selected_idx];
        let (manager_statement, impact) = trial_respond(&mut session, corpus, selected_option_id);

        let manager_speeches: Vec<_> = manager_statement
            .speech_sequence
            .iter()
            .map(|s| SpeechSnapshot {
                id: s.id,
                summary: Some(corpus.responses[s.id as usize].summary.clone()),
                text: s.text.clone(),
                emotion: s.appearance.emotion,
                gesture: s.appearance.gesture,
                topic: None,
            })
            .collect();

        let impact_snap = ImpactSnapshot {
            reputation_score: impact.reputation.response_score,
            mood_change: impact.psych.mood_change,
            trust_change: impact.psych.trust_change,
            patience_change: impact.psych.patience_change,
        };

        let should_continue = trial_should_continue(&mut session, corpus);

        snapshot.turns.push(TurnSnapshot {
            turn_number: turn_num,
            diner_speeches,
            response_options: Some(all_speech_options),
            selected_response: Some(ResponseSelection {
                speech_id: selected_speech_id,
                keyword_idx: selected_keyword_idx,
                option_idx: selected_idx as SpeechId,
            }),
            manager_speeches: Some(manager_speeches),
            impact: Some(impact_snap),
            continuation_decision: Some(should_continue),
        });

        if !should_continue {
            break;
        }
    }

    snapshot.final_state = FinalState {
        total_turns: snapshot.turns.len(),
        asked_questions: session.get_asked_questions().to_vec(),
        continuation_depth: session.continuation_depth,
    };

    snapshot
}

#[test]
fn test_trial_snapshots() {
    use rand::prelude::*;

    // Generate 20 diverse trial scenarios with different seeds and topics
    let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42);

    let topics = [
        FeedbackTopic::Appeal,
        FeedbackTopic::Queue,
        FeedbackTopic::Tableware,
        FeedbackTopic::Quality,
        FeedbackTopic::Price,
        FeedbackTopic::Hygiene,
        FeedbackTopic::Taste,
        FeedbackTopic::Hunger,
        FeedbackTopic::Praise,
        FeedbackTopic::Crab,
    ];

    for i in 0..topics.len() * 2 + 5 {
        let seed = rng.random();
        let topic = if i / 2 < topics.len() {
            Some(topics[i / 2])
        } else {
            None
        };

        let snapshot = run_trial_snapshot(seed, topic);

        // Create unique snapshot name for each iteration
        let topic_name = topic
            .map(|t| format!("{:?}", t).to_lowercase())
            .unwrap_or_else(|| "".to_string());
        let snapshot_name = format!("trial_{topic_name}_{seed:016x}");
        insta::with_settings!({prepend_module_to_snapshot => false, omit_expression => true}, {
            insta::assert_yaml_snapshot!(snapshot_name, snapshot);
        });
    }
}
