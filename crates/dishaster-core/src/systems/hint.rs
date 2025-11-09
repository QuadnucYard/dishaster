use super::prelude::*;

pub mod hints {
    pub const DISPENSER_OUT_OF_STOCK: &str = "dispenser-out-of-stock";
}

pub trait HintEmitter {
    fn emit_hint(&mut self, hint_id: impl Into<EcoString>);
}

impl HintEmitter for EventQueue {
    fn emit_hint(&mut self, hint_id: impl Into<EcoString>) {
        self.push(SimEvent::ShowHint(hint_id.into()));
    }
}
