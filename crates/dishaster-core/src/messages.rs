use dishaster_models::FeedbackTrigger;
use dishaster_views::Feedback;

use crate::prelude::*;

#[derive(Message)]
pub struct FeedbackMessage {
    /// Entity currently expressing the feedback.
    pub entity: Entity,
    /// Content of the feedback.
    pub content: Feedback,
    /// Optional trigger type for more specific feedback handling (for trial system)
    #[allow(dead_code)]
    pub trigger: Option<FeedbackTrigger>,
}

#[derive(Message)]
pub struct RefillDispenser(pub Entity);
