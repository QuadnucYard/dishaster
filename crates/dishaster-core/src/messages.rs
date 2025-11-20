use dishaster_models::{FeedbackTopic, Seconds};
use dishaster_views::Feedback;

use crate::prelude::*;

/// Message conveying feedback from an entity.
#[derive(Message)]
pub struct FeedbackMessage {
    /// Entity currently expressing the feedback.
    pub entity: Entity,
    /// Content of the feedback.
    pub content: Feedback,
    /// Optional trigger type for more specific feedback handling (for trial system)
    pub trigger: Option<FeedbackTopic>,
    /// Duration in seconds to display this feedback.
    pub display_duration: f32,
}

/// Request to refill a dispenser entity.
#[derive(Message)]
pub struct RefillDispenser(pub Entity);

/// Notification that a dish has been served to a diner.
#[derive(Message)]
pub struct DishServed {
    /// Diner entity receiving the dish.
    pub diner: Entity,
    /// Identifier of the dish model served.
    pub dish_id: ModelId,
    /// Time taken to serve the dish (seconds).
    pub service_time: Seconds,
    /// Price charged for the served dish.
    pub price: f32,
    /// Weight of the served portion (kg).
    pub weight: f32,
    /// Quality level of the served dish.
    pub quality: f32,
    /// Any contamination level of the served dish.
    pub contamination: f32,
}

/// Notification that a queue service completion occurred.
/// Used to update queue service history for wait time estimation.
#[derive(Message)]
pub struct QueueServiceCompleted {
    /// Lane where the service was completed.
    pub lane: Entity,
}
