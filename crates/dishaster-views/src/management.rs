use dishrupt_core::prelude::*;

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
    // /// Icon identifier for visual representation
    // pub icon: EcoString,
    // /// List of effect descriptions to display (from l10n: `{decision_id}-effects`)
    // pub effects: Vec<EcoString>,
    // /// Theme hint for styling ("upgrade", "risk", "neutral")
    // pub theme: EcoString,
}

/// View model for management incident notification at day start

#[derive(Debug, Clone)]
pub struct ManagementIncidentView {
    /// Incident identifier (used to look up l10n keys)
    pub incident_id: ModelId,
    // /// Icon identifier for visual representation
    // pub icon: EcoString,
    // /// List of effect descriptions to display (from l10n: `{incident_id}-effects`)
    // pub effects: Vec<EcoString>,
    // /// Theme hint for styling ("warning", "info", "challenge")
    // pub theme: EcoString,
}
