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

#[derive(Event)]
pub struct ApplyTrialImpact {
    pub diner: Entity,
    pub psych_impact: PsychImpact,
    pub reputation_impact: ReputationImpact,
}
