use dishrupt_core::prelude::*;
use rustc_hash::FxHashSet;

/// Hint system for tracking first-time event guidance
///
/// Manages which tutorial hints have been shown to the player
/// to avoid repeating the same guidance across sessions.
#[derive(Default)]
pub struct HintTracker {
    /// Set of hint IDs that have been shown during this session
    shown_hints: FxHashSet<EcoString>,
}

impl HintTracker {
    /// Create a new hint tracker with pre-shown hints
    pub fn new(shown_hints: FxHashSet<EcoString>) -> Self {
        Self { shown_hints }
    }

    /// Mark a hint as shown. Returns true if it was not shown before.
    pub fn mark_shown(&mut self, hint_id: &str) -> bool {
        self.shown_hints.insert(hint_id.into())
    }
}
