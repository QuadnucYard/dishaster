use dishrupt_core::prelude::EcoString;

use super::prelude::*;

/// The corpus of trial speeches and responses.
#[derive(Debug, Default)]
pub struct TrialCorpus {
    /// Speeches made by the diner (left participant).
    pub diner_speeches: Vec<TrialSpeech>,
    /// Possible responses made by the player (right participant).
    pub responses: Vec<TrialResponse>,
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

/// Introduction data for a trial.
#[derive(Debug)]
pub struct TrialIntro {
    /// Appearance of the left participant.
    pub left: TrialParticipantAppearance,
    /// Appearance of the right participant.
    pub right: TrialParticipantAppearance,
}

/// A statement made during a trial, along with possible response options.
#[derive(Debug)]
pub struct TrialStatement {
    /// The speech made by the left participant.
    pub speech: TrialSpeech,
    /// Possible response options for each keyword.
    pub options: Vec<Vec<TrialResponseOption>>,
}

/// A single response option during a trial.
#[derive(Debug)]
pub struct TrialResponseOption {
    /// Index into the corpus responses.
    pub corpus_index: usize,
    /// The kind of trial response.
    pub kind: TrialResponseKind,
    /// A brief summary of the response, displayed in the options list.
    pub summary: EcoString,
}
