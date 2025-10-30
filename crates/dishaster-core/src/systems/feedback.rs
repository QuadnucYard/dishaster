#![allow(missing_docs)]

use dishaster_views::FeedbackView;

use super::prelude::*;

// For now, we ue simple emoji strings as feedback indicators.
// These are converted to graphical balloons in the client.
pub const OBSERVING_FEEDBACKS: &[&str] = &["👀", "🤔", "📝"];
pub const DECIDING_FEEDBACKS: &[&str] = &["😋", "😕", "💡"];
pub const SERVING_FEEDBACKS: &[&str] = &["🍚", "🍲", "👍", "❓"];

pub fn choose_feedback<'a>(rng: &mut impl Rng, pool: &'a [&str]) -> &'a str {
    pool.choose(rng).expect("pool is non-empty")
}

impl EventQueue {
    pub fn emit_feedback(&mut self, event: FeedbackView) {
        self.push(SimEvent::Feedback(event));
    }
}
