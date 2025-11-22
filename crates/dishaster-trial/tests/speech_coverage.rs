//! Integration test for trial system speech coverage.
//!
//! This test verifies that most trial speeches can be hit through various trial sessions,
//! ensuring the corpus is well-utilized and diverse dialogue paths are accessible.

use std::collections::HashSet;

use dishaster_data::DataLoader;
use dishaster_models::FeedbackTopic;
use dishaster_trial::{
    TrialSession, create_diner_statement, create_trial_intro, generate_trial_response_candidates,
    trial_respond, trial_should_continue,
};
use dishaster_views::SpeechId;
use dishrupt_core::EntityId;
use dishrupt_rng::prelude::*;

/// Statistics for trial speech coverage
#[derive(Debug, Default)]
struct CoverageStats {
    /// Diner speeches (questions) that were used
    used_diner_speeches: HashSet<SpeechId>,
    /// Player responses that were offered as options
    offered_responses: HashSet<SpeechId>,
    /// Player responses that were actually selected
    selected_responses: HashSet<SpeechId>,
    /// Total number of trial sessions run
    total_trials: usize,
    /// Total number of turns across all trials
    total_turns: usize,
}

impl CoverageStats {
    fn new() -> Self {
        Self::default()
    }

    fn add_diner_speech(&mut self, speech_id: SpeechId) {
        self.used_diner_speeches.insert(speech_id);
    }

    fn add_offered_responses(&mut self, response_ids: &[SpeechId]) {
        self.offered_responses.extend(response_ids);
    }

    fn add_selected_response(&mut self, response_id: SpeechId) {
        self.selected_responses.insert(response_id);
    }

    fn report(&self, total_diner_speeches: usize, total_responses: usize) {
        let diner_coverage =
            (self.used_diner_speeches.len() as f32 / total_diner_speeches as f32) * 100.0;
        let response_offer_coverage =
            (self.offered_responses.len() as f32 / total_responses as f32) * 100.0;
        let response_selection_coverage =
            (self.selected_responses.len() as f32 / total_responses as f32) * 100.0;

        println!("\n=== Trial Speech Coverage Report ===");
        println!(
            "Diner speeches used: {} / {} ({:.1}%)",
            self.used_diner_speeches.len(),
            total_diner_speeches,
            diner_coverage
        );
        println!(
            "Responses offered: {} / {} ({:.1}%)",
            self.offered_responses.len(),
            total_responses,
            response_offer_coverage
        );
        println!(
            "Responses selected: {} / {} ({:.1}%)",
            self.selected_responses.len(),
            total_responses,
            response_selection_coverage
        );
        println!("Total trials run: {}", self.total_trials);
        println!(
            "Total turns: {} (avg {:.1} per trial)",
            self.total_turns,
            self.total_turns as f32 / self.total_trials.max(1) as f32
        );
        println!();

        // Verify coverage thresholds
        // Note: Thresholds are realistic based on dialogue structure:
        // - Not all speeches are reachable from all topics
        // - Some responses are specific to certain contexts
        assert!(
            diner_coverage >= 70.0,
            "Diner speech coverage {:.1}% is below 70% threshold",
            diner_coverage
        );
        assert!(
            response_offer_coverage >= 60.0,
            "Response offer coverage {:.1}% is below 60% threshold",
            response_offer_coverage
        );
        assert!(
            response_selection_coverage >= 40.0,
            "Response selection coverage {:.1}% is below 40% threshold",
            response_selection_coverage
        );
    }
}

/// Run a single trial session and collect coverage statistics
fn run_trial_session(
    session: &mut TrialSession,
    corpus: &dishaster_models::TrialCorpus,
    target_entity: EntityId,
    topic: Option<FeedbackTopic>,
    rng: &mut impl Rng,
    stats: &mut CoverageStats,
) {
    session.start(target_entity, topic);
    stats.total_trials += 1;

    let _intro = create_trial_intro(session);

    // Run trial until completion (max 20 turns)
    'trial_loop: for _ in 0..20 {
        // Get diner statement
        let statement = create_diner_statement(session, corpus);

        // Track all diner speeches used
        for speech in &statement.speech_sequence {
            stats.add_diner_speech(speech.id);
        }

        let mut responded = false;

        // For each keyword in each speech, generate response candidates
        'speech_loop: for speech in &statement.speech_sequence {
            for (keyword_idx, item) in speech.items.iter().enumerate() {
                if matches!(item, dishaster_views::TrialSpeechItem::Keyword(_)) {
                    let options =
                        generate_trial_response_candidates(session, corpus, speech.id, keyword_idx);

                    let option_ids: Vec<_> = options.iter().map(|opt| opt.id).collect();
                    stats.add_offered_responses(&option_ids);

                    // Randomly select a response or skip (simulating player behavior)
                    let response_chance = rng.random_range(0.0..1.0);
                    if response_chance > 0.1 && !options.is_empty() {
                        // 90% chance to respond
                        let selected_idx = rng.random_range(0..options.len());
                        let selected_id = options[selected_idx].id;
                        stats.add_selected_response(selected_id);

                        // Apply the response
                        let (_response_statement, _impact) =
                            trial_respond(session, corpus, selected_id);
                        stats.total_turns += 1;
                        responded = true;

                        // Check if trial should continue
                        let should_continue = trial_should_continue(session, corpus);
                        if !should_continue {
                            return; // Trial ended
                        }

                        // Responded, continue to next turn
                        break 'speech_loop;
                    }
                }
            }
        }

        // If we didn't respond to any keyword, trial times out
        if !responded {
            stats.total_turns += 1;
            break 'trial_loop;
        }
    }
}

#[test]
fn coverage() {
    println!("Loading trial corpus...");
    let mut loader = DataLoader::from_fs("../../assets/data").unwrap();
    let data = loader.load_all_data().unwrap();
    let corpus = &data.models.trial;

    let total_diner_speeches = corpus.diner_speeches.len();
    let total_responses = corpus.responses.len();

    println!(
        "Corpus loaded: {} diner speeches, {} responses",
        total_diner_speeches, total_responses
    );

    let mut stats = CoverageStats::new();
    let mut rng = Prng::new(42);

    // Run multiple trial sessions with different configurations
    let num_trials = 1000;
    let topics = [
        None,
        Some(FeedbackTopic::Appeal),
        Some(FeedbackTopic::Queue),
        Some(FeedbackTopic::Tableware),
        Some(FeedbackTopic::Quality),
        Some(FeedbackTopic::Price),
        Some(FeedbackTopic::Hygiene),
        Some(FeedbackTopic::Taste),
        Some(FeedbackTopic::Hunger),
        Some(FeedbackTopic::Crab),
    ];

    println!("Running {} trial sessions...", num_trials);
    for i in 0..num_trials {
        if (i + 1) % 10 == 0 {
            println!("  Progress: {}/{}", i + 1, num_trials);
        }

        let seed: u64 = rng.random();
        let mut session = TrialSession::new(seed);
        let target_entity = EntityId::new(seed % 100 + 1).unwrap();
        let topic = topics[i % topics.len()];

        run_trial_session(
            &mut session,
            corpus,
            target_entity,
            topic,
            &mut rng,
            &mut stats,
        );
    }

    // Report coverage statistics
    stats.report(total_diner_speeches, total_responses);

    // Print sample of covered speeches
    println!("Sample of used diner speeches (first 10):");
    for (i, &speech_id) in stats.used_diner_speeches.iter().take(10).enumerate() {
        let speech = &corpus.diner_speeches[speech_id as usize];
        let text = speech.text.chars().take(50).collect::<String>();
        println!("  {}. Speech {}: {}...", i + 1, speech_id, text);
    }

    println!("\nSample of offered responses (first 10):");
    for (i, &response_id) in stats.offered_responses.iter().take(10).enumerate() {
        let response = &corpus.responses[response_id as usize];
        println!(
            "  {}. Response {}: {}",
            i + 1,
            response_id,
            response.summary
        );
    }

    println!("\n=== Coverage Analysis ===");
    println!(
        "Unused diner speeches: {} (may be topic-specific or hard to reach)",
        total_diner_speeches - stats.used_diner_speeches.len()
    );
    println!(
        "Unopposed responses: {} (very specific or low relevance)",
        total_responses - stats.offered_responses.len()
    );
    println!("\n✓ Trial coverage test passed!\n  Most dialogue paths are accessible and diverse.");
}
