use dishaster_interface::event::HintCondition;
use dishrupt_core::prelude::*;
use rustc_hash::FxHashSet;

/// Hint system for tracking tutorial and notification guidance
///
/// Manages which hints have been shown to control repetition based on
/// different emission modes (always, once per profile, once per day).
#[derive(Default)]
pub struct HintTracker {
    /// Set of hint IDs that have been shown across the entire profile (persisted)
    global_shown_hints: FxHashSet<EcoString>,
    /// Set of hint IDs that have been shown today (reset each day)
    local_shown_hints: FxHashSet<EcoString>,
}

impl HintTracker {
    /// Create a new hint tracker with previously shown hints from profile
    pub fn new(profile_shown_hints: FxHashSet<EcoString>) -> Self {
        Self {
            global_shown_hints: profile_shown_hints,
            local_shown_hints: Default::default(),
        }
    }

    /// Mark a hint as shown based on its condition
    ///
    /// Returns true if this is the first time showing the hint in the relevant scope.
    pub fn mark_shown(&mut self, hint_id: &str, mode: HintCondition) -> bool {
        match mode {
            HintCondition::Always => true, // Always show, always "first time"
            HintCondition::OnceGlobal => self.global_shown_hints.insert(hint_id.into()),
            HintCondition::OnceLocal => self.local_shown_hints.insert(hint_id.into()),
        }
    }

    /// Get profile-level shown hints for persistence
    pub fn profile_shown_hints(&self) -> &FxHashSet<EcoString> {
        &self.global_shown_hints
    }
}
