use dishrupt_core::prelude::*;

use crate::ParamsMap;

/// View model for management decisions
#[derive(Debug, Clone)]
pub struct ManagementDecisionsView {
    /// Current day
    pub day: u32,
    /// Available management decisions
    pub options: Vec<ManagementDecisionView>,
}

/// View model for management decision card
#[derive(Debug, Clone)]
pub struct ManagementDecisionView {
    /// Decision identifier (used to look up l10n keys)
    pub model_id: ModelId,
    /// Icon identifier for visual representation
    pub icon: SpriteRef,
    // /// Theme hint for styling ("upgrade", "risk", "neutral")
    // pub theme: EcoString,
    /// Parameters for localization formatting
    pub params: ParamsMap,
}

/// View model for management incident notification at day start

#[derive(Debug, Clone)]
pub struct ManagementIncidentView {
    /// Incident identifier (used to look up l10n keys)
    pub model_id: ModelId,
    /// Icon identifier for visual representation
    pub icon: SpriteRef,
    // /// Theme hint for styling ("warning", "info", "challenge")
    // pub theme: EcoString,
    /// Parameters for localization formatting
    pub params: ParamsMap,
}

/// View model for inspector visit result display (when passed)
#[derive(Debug, Clone)]
pub struct InspectorResultView {
    /// Reputation boost if passed (0 if failed)
    pub reputation_boost: f32,
    /// Trust boost if passed (0 if failed)
    pub trust_boost: f32,
}
