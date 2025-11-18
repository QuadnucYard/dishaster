//! The trial system. It works outside the ECS loop.

mod adapter;
mod speech;

use dishaster_models::{FeedbackTopic, TrialConfig};
use dishaster_views::{SpeechId, TrialResponseOption};
use dishrupt_core::EntityId;

pub use self::speech::*;
use crate::prelude::*;

mod prelude {
    pub use dishrupt_rng::prelude::*;
    pub use rustc_hash::{FxHashMap, FxHashSet};
}

/// Combined impact of a trial response on both reputation and diner psychology.
///
/// This structure encapsulates how a manager's response during a trial affects
/// both the restaurant's overall reputation and the specific diner's psychological state.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrialImpact {
    /// Impact on the restaurant's global reputation score
    pub reputation: ReputationImpact,
    /// Impact on the diner's psychological state (mood, trust, patience)
    pub psych: PsychImpact,
}

/// Impact on the restaurant's global reputation score.
///
/// This affects how the restaurant is perceived by all potential customers.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReputationImpact {
    /// Response quality score, typically ranging from -1.0 (poor) to 1.0 (excellent)
    pub response_score: f32,
}

/// Impact on a specific diner's psychological state during a trial.
///
/// These values affect the diner's behavior and decision-making throughout their visit.
/// Values are unclamped and will be applied with appropriate constraints by the system.
#[derive(Debug, Clone, Copy, Default)]
pub struct PsychImpact {
    /// Change in the diner's emotional state (positive = happier, negative = angrier)
    pub mood_change: f32,
    /// Change in the diner's trust in restaurant management
    pub trust_change: f32,
    /// Change in the diner's tolerance for waiting (affects queue behavior)
    pub patience_change: f32,
}

/// Trial session state tracking to avoid repetition and improve coherence
pub struct TrialSession {
    /// Configuration parameters for trial behavior
    pub config: TrialConfig,
    /// Pseudorandom number generator for trial session
    pub rng: Prng,
    /// Whether the trial has ever been triggered in this run
    pub ever_triggered: bool,
    /// The diner entity that triggered this trial (for applying psych state impacts)
    pub target_entity: Option<EntityId>,
    /// Indices of questions already asked in this trial
    asked_questions: Vec<SpeechId>,
    /// Cached response options for (speech_id, keyword_index) pairs
    pub cached_options: FxHashMap<(SpeechId, usize), Vec<TrialResponseOption>>,
    /// The most recent response corpus index selected by the player
    pub last_response_id: Option<SpeechId>,
    /// The most recent diner speech index
    pub last_diner_speech_id: Option<SpeechId>,
    /// The most recent diner speech index that the player is responding to (for context evaluation)
    pub current_question_id: Option<SpeechId>,
    /// Current continuation depth (consecutive speeches by same speaker)
    pub continuation_depth: u32,
    /// Maximum allowed continuation depth before forcing speaker alternation
    pub max_continuation_depth: u32,
    /// Temperature parameter for sampling (higher = more random, lower = more deterministic)
    pub temperature: f32,
    /// Topic that triggered this trial (for filtering relevant speeches)
    pub trigger_topic: Option<FeedbackTopic>,
}

impl TrialSession {
    /// Create a new trial session with default temperature
    pub fn new(seed: u64) -> Self {
        Self {
            config: TrialConfig::default(),
            rng: Prng::new(seed),
            ever_triggered: false,
            target_entity: None,
            asked_questions: Vec::new(),
            cached_options: Default::default(),
            last_response_id: None,
            last_diner_speech_id: None,
            current_question_id: None,
            continuation_depth: 0,
            max_continuation_depth: 3,
            temperature: 0.8,
            trigger_topic: None,
        }
    }

    /// Reset the session for a new trial
    pub fn reset(&mut self) {
        self.target_entity = None;
        self.asked_questions.clear();
        self.cached_options.clear();
        self.last_response_id = None;
        self.last_diner_speech_id = None;
        self.current_question_id = None;
        self.continuation_depth = 0;
        self.trigger_topic = None;
    }

    /// Starts a new trial session with the specified diner and optional topic filter.
    ///
    /// This resets all session state except the RNG and sets up for a new trial.
    /// If a topic is provided, only speeches related to that topic will be generated.
    pub fn start(&mut self, target: EntityId, topic: Option<FeedbackTopic>) {
        self.reset();
        self.ever_triggered = true;
        self.target_entity = Some(target);
        self.trigger_topic = topic;
    }

    /// Check if a question has been asked
    pub fn has_asked(&self, question_id: u32) -> bool {
        self.asked_questions.contains(&question_id)
    }

    /// Get the list of asked question indices
    pub fn get_asked_questions(&self) -> &[u32] {
        &self.asked_questions
    }

    /// Mark a question as asked
    pub fn mark_asked(&mut self, question_id: SpeechId) {
        if !self.has_asked(question_id) {
            self.asked_questions.push(question_id);
        }
    }

    /// Record the player's response choice
    pub fn set_last_response(&mut self, response_id: SpeechId) {
        self.last_response_id = Some(response_id);
        // Reset continuation depth when speaker alternates
        self.continuation_depth = 0;
    }

    /// Record a diner speech
    pub fn set_last_diner_speech(&mut self, speech_id: SpeechId) {
        self.last_diner_speech_id = Some(speech_id);
    }

    /// Set the current question index (the question player is responding to)
    pub fn set_current_question(&mut self, question_id: SpeechId) {
        self.current_question_id = Some(question_id);
    }

    /// Increment continuation depth
    #[allow(unused)]
    pub fn increment_continuation(&mut self) {
        self.continuation_depth += 1;
    }

    /// Check if continuation is allowed (not at max depth)
    pub fn can_continue(&self) -> bool {
        self.continuation_depth < self.max_continuation_depth
    }

    /// Reset continuation depth (when alternating speakers)
    pub fn reset_continuation(&mut self) {
        self.continuation_depth = 0;
    }

    /// Decide whether to continue based on best continuation score
    /// Uses the score as probability (higher score = more likely to continue)
    pub fn should_continue(&mut self, best_score: f32) -> bool {
        if !self.can_continue() {
            return false;
        }

        // Use score directly as probability (already normalized 0-1 from embedding similarity)
        // Low scores (<0.3) rarely continue, high scores (>0.7) usually continue
        let prob = best_score.clamp(0.0, 1.0);
        self.rng.random_bool(prob as f64)
    }
}
