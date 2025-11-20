use dishrupt_core::prelude::*;

use super::{FeedbackTopic, prelude::*};

/// Configuration parameters for trial speech generation and impact calculations.
///
/// These values control various aspects of trial behavior including:
/// - Speech continuation probabilities and multipliers
/// - Relevance scoring thresholds
/// - Psychological impact scaling factors
/// - Timeout penalties
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrialConfig {
    /// Multiplier for RR (response-response) continuation scores.
    /// Lower values make manager responses more concise.
    pub rr_continuation_multiplier: f32,

    /// Multiplier for QQ (question-question) continuation scores.
    /// Lower values make diner speeches more concise.
    pub qq_continuation_multiplier: f32,

    /// Base probability for continuing dialogue when no rank data available.
    pub no_rank_continuation_prob: f32,

    /// Score threshold for highly relevant responses (no penalty).
    pub relevance_high_threshold: f32,

    /// Score threshold for somewhat relevant responses (small penalty).
    pub relevance_medium_threshold: f32,

    /// Penalty for somewhat relevant responses.
    pub relevance_medium_penalty: f32,

    /// Penalty for irrelevant responses.
    pub relevance_low_penalty: f32,

    /// Response score used when trial times out.
    pub timeout_response_score: f32,

    /// Mood change when trial times out.
    pub timeout_mood_penalty: f32,

    /// Trust change when trial times out.
    pub timeout_trust_penalty: f32,

    /// Patience change when trial times out.
    pub timeout_patience_penalty: f32,

    /// Scaling factor for mood change based on response score.
    pub mood_scale: f32,

    /// Scaling factor for trust change based on response score.
    pub trust_scale: f32,

    /// Scaling factor for patience change based on response score.
    pub patience_scale: f32,

    /// Minimum time interval in seconds between trials (cooldown).
    pub trial_cooldown_seconds: f32,
}

impl Default for TrialConfig {
    fn default() -> Self {
        Self {
            rr_continuation_multiplier: 0.2,
            qq_continuation_multiplier: 0.3,
            no_rank_continuation_prob: 0.3,
            relevance_high_threshold: 0.8,
            relevance_medium_threshold: 0.6,
            relevance_medium_penalty: -0.2,
            relevance_low_penalty: -0.5,
            timeout_response_score: -0.8,
            timeout_mood_penalty: -0.15,
            timeout_trust_penalty: -0.1,
            timeout_patience_penalty: -5.0,
            mood_scale: 0.1,
            trust_scale: 0.05,
            patience_scale: 2.0,
            trial_cooldown_seconds: 5.0,
        }
    }
}

/// The corpus of trial speeches and responses.
#[derive(Debug, Default)]
pub struct TrialCorpus {
    /// Speeches made by the diner (left participant).
    pub diner_speeches: Vec<TrialSpeech>,
    /// Possible responses made by the player (right participant).
    pub responses: Vec<TrialResponse>,

    /// Question->Answer ranks for trial speeches.
    pub qa_ranks: Vec<Vec<Vec<TrialRank>>>,
    /// Answer->Question ranks for trial speeches.
    pub aq_ranks: Vec<Vec<TrialRank>>,
    /// Question->Question continuation ranks (for multi-turn diner speeches).
    pub qq_ranks: Vec<Vec<TrialRank>>,
    /// Response->Response continuation ranks (for multi-turn player responses).
    pub rr_ranks: Vec<Vec<TrialRank>>,
}

/// A single speech made by a trial participant.
#[derive(Debug, Clone, Deserialize)]
pub struct TrialSpeech {
    /// The text of the statement.
    pub text: EcoString,
    /// The breakdown of the statement into items.
    #[serde(skip)]
    pub items: Vec<TrialSpeechItem>,
    /// The appearance associated with the statement.
    #[serde(flatten)]
    pub appearance: TrialParticipantAppearance,
    /// Optional topic that this speech addresses (for trial triggering from feedback).
    /// If present, this speech can only be triggered by feedback with matching topic.
    #[serde(default)]
    pub topic: Option<FeedbackTopic>,
}

/// An item within a trial speech.
#[derive(Debug, Clone, PartialEq)]
pub enum TrialSpeechItem {
    /// Plain text.
    Text(EcoString),
    /// A keyword emphasized in the speech.
    Keyword(EcoString),
    /// A line break in the speech.
    LineBreak,
}

impl TrialSpeech {
    /// Iterator over keywords in the speech.
    pub fn keywords(&self) -> impl Iterator<Item = &TrialSpeechItem> {
        self.items
            .iter()
            .filter(|item| matches!(item, TrialSpeechItem::Keyword(_)))
    }
}

/// The appearance of a trial participant.
#[derive(Debug, Clone, Deserialize)]
pub struct TrialParticipantAppearance {
    /// The emoji of the statement.
    pub emotion: char,
    /// Optional gesture associated with the response.
    pub gesture: Option<char>,
}

/// A possible response during a trial.
#[derive(Debug, Clone, Deserialize)]
pub struct TrialResponse {
    /// The kind of trial response.
    pub kind: TrialResponseKind,
    /// A brief summary of the response.
    pub summary: EcoString,
    /// The content of the response.
    #[serde(flatten)]
    pub content: TrialSpeech,
    /// Quality score of this response [-1.0, 1.0]
    /// Positive values indicate helpful/diplomatic responses
    /// Negative values indicate confrontational/poor responses
    /// Used to modify reputation impact from feedback
    #[serde(default)]
    pub response_score: f32,
}

/// The kind of trial response.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialResponseKind {
    Agreement,
    Objection,
    Perjury,
    Question,
}

/// A single trial rank entry.
#[derive(Clone, Copy, bincode::Decode)]
pub struct TrialRank {
    /// Index of the answer in the corpus.
    pub answer_id: u32,
    /// The score of the answer.
    pub score: f32,
}

impl std::fmt::Display for TrialRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.answer_id, self.score)
    }
}

impl std::fmt::Debug for TrialRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
