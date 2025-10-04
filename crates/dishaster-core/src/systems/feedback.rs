#![allow(missing_docs)]

use super::prelude::*;

// For now, we ue simple emoji strings as feedback indicators.
// These are converted to graphical balloons in the client.
pub const OBSERVING_FEEDBACKS: &[&str] = &["👀", "🤔", "📝"];
pub const DECIDING_FEEDBACKS: &[&str] = &["😋", "😕", "💡"];
pub const SERVING_FEEDBACKS: &[&str] = &["🍚", "🍲", "👍", "❓"];

pub fn choose_feedback<'a>(rng: &mut GameRng, pool: &'a [&str]) -> &'a str {
    pool.choose(rng).expect("pool is non-empty")
}

impl EventLog {
    pub fn emit_feedback(&mut self, event: FeedbackEvent) {
        self.0.push(PresentationEvent::Feedback(event));
    }
}
