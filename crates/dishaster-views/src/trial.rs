use dishrupt_core::prelude::*;

/// Unique identifier for a speech in the trial corpus.
pub type SpeechId = u32;

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
    /// The sequence of speeches made by the left participant (diner).
    /// Displayed with delays; player can interrupt to respond early.
    pub speech_sequence: Vec<TrialSpeech>,
}

/// A single speech made by a trial participant.
#[derive(Debug)]
pub struct TrialSpeech {
    /// The index of the speech in the corpus.
    pub id: SpeechId,
    /// The text of the statement.
    pub text: EcoString,
    /// The breakdown of the statement into items.
    pub items: Vec<TrialSpeechItem>,
    /// The appearance associated with the statement.
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

/// A single response option during a trial.
#[derive(Debug, Clone)]
pub struct TrialResponseOption {
    /// Index into the corpus responses.
    pub id: SpeechId,
    /// The kind of trial response.
    pub kind: TrialResponseKind,
    /// A brief summary of the response, displayed in the options list.
    pub summary: EcoString,
}

/// A possible response during a trial.
#[derive(Debug)]
pub struct TrialResponse {
    /// The kind of trial response.
    pub kind: TrialResponseKind,
    /// A brief summary of the response.
    pub summary: EcoString,
    /// The content of the response.
    pub content: TrialSpeech,
}

/// The kind of trial response.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialResponseKind {
    Agreement,
    Objection,
    Perjury,
    Question,
}

/// The appearance of a trial participant.
#[derive(Debug, Clone)]
pub struct TrialParticipantAppearance {
    /// The emoji of the statement.
    pub emotion: char,
    /// Optional gesture associated with the response.
    pub gesture: Option<char>,
}
