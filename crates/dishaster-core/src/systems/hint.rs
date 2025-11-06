use super::prelude::*;

pub mod hints {
    pub const DISPENSER_OUT_OF_STOCK: &str = "dispenser_out_of_stock";
}

/// Emit a hint event if this is the first time the player encounters it
pub fn emit_hint_if_first_time(hints: &mut HintTracker, events: &mut EventQueue, hint_id: &str) {
    if !hints.has_shown(hint_id) {
        hints.mark_shown(hint_id);
        events.push(SimEvent::ShowHint(hint_id.into()));
    }
}
