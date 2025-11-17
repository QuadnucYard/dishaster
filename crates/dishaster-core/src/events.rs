use dishaster_models::{EndingType, FeedbackTopic, InspectorVisitModel};
use dishaster_trial::{PsychImpact, ReputationImpact};

use crate::prelude::*;

/// Event signaling the start of a run
#[derive(Event)]
pub struct RunStarted;

/// Event signaling the end of a run
#[derive(Event)]
pub struct RunEnded;

/// Event to advance to the next day
#[derive(Event)]
pub struct AdvanceDay;

/// Event to achieve a game ending
#[derive(Event)]
pub struct AchieveEnding(pub EndingType);

/// Event to roll management decisions
#[derive(Event)]
pub struct RollManagementDecisions;

/// Event to apply a selected management decision
#[derive(Event)]
pub struct ApplyManagementDecision(pub usize);

/// Event to roll a management incident
#[derive(Event)]
pub struct RollManagementIncident;

/// Event to dispatch a selected management decision
#[derive(Event)]
pub struct DispatchManagement<T>(pub T);

/// Event to trigger an inspector visit (can be from incident, dev command, etc.)
#[derive(Event)]
pub struct InspectorVisit(pub InspectorVisitModel);

/// Event signaling the start of a trial
#[derive(Event)]
pub struct TrialStart {
    pub diner: Entity,
    pub topic: Option<FeedbackTopic>,
}

/// Event to launch the trial after intro is complete
#[derive(Event)]
pub struct TrialLaunch;

/// Event to choose a response during the trial
#[derive(Event)]
pub struct TrialRespond(pub usize);

/// Event to timeout the current trial response
#[derive(Event)]
pub struct TrialTimeout;

/// Event to request response candidates for a specific keyword (lazy loading)
#[derive(Event)]
pub struct TrialRequestCandidates {
    pub speech_id: usize,
    pub keyword_index: usize,
}

/// Event to proceed to the next dialogue of the trial
#[derive(Event)]
pub struct TrialProceed;

#[derive(Event)]
pub struct ApplyTrialImpact {
    pub diner: Entity,
    pub psych_impact: PsychImpact,
    pub reputation_impact: ReputationImpact,
}
