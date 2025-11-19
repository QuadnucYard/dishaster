use super::prelude::*;

pub mod hints {
    /// Hint shown when dispenser runs out of stock.
    /// Emission: Always (critical warning)
    pub const DISPENSER_OUT_OF_STOCK: &str = "dispenser-out-of-stock";

    /// Hint shown when feedback can trigger a trial.
    /// Emission: Once per profile (first-time tutorial)
    pub const CLICK_FEEDBACK_TO_TRIAL: &str = "click-feedback-to-trial";

    /// Hint shown in preparation phase about price adjustment.
    /// Emission: Once per day (daily reminder)
    pub const ADJUST_PRICE: &str = "adjust-price";

    /// Hint shown when refill staff limit reached and cannot spawn more.
    /// Emission: Always (operational limit warning)
    pub const REFILL_STAFF_BUSY: &str = "refill-staff-busy";
}

pub trait HintEmitter {
    fn emit_hint(&mut self, hint_id: impl Into<EcoString>, cond: HintCondition);
}

impl HintEmitter for EventQueue {
    fn emit_hint(&mut self, hint_id: impl Into<EcoString>, condition: HintCondition) {
        self.push(SimEvent::ShowHint {
            id: hint_id.into(),
            condition,
        });
    }
}
