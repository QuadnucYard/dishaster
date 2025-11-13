use dishrupt_core::prelude::*;

use super::prelude::*;

/// The corpus of trial speeches and responses.
#[derive(Debug, Default)]
pub struct TrialCorpus {
    /// Speeches made by the diner (left participant).
    pub diner_speeches: Vec<TrialSpeech>,
    /// Possible responses made by the player (right participant).
    pub responses: Vec<TrialResponse>,

    /// Question->Answer ranks for trial speeches.
    pub qa_ranks: Vec<Vec<Vec<TrialQARank>>>,
    /// Answer->Question ranks for trial speeches.
    pub aq_ranks: Vec<Vec<TrialQARank>>,
    /// Question->Question continuation ranks (for multi-turn diner speeches).
    pub qq_ranks: Vec<Vec<TrialQARank>>,
    /// Response->Response continuation ranks (for multi-turn player responses).
    pub rr_ranks: Vec<Vec<TrialQARank>>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialResponseKind {
    Agreement,
    Objection,
    Perjury,
    Question,
}

/// A single QA rank entry.
#[derive(Clone, Copy)]
pub struct TrialQARank {
    /// Index of the answer in the corpus.
    pub answer_index: usize,
    /// The score of the answer.
    pub score: f32,
}

impl std::fmt::Display for TrialQARank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.answer_index, self.score)
    }
}

impl std::fmt::Debug for TrialQARank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
