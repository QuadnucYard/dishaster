use dishaster_interface::*;

use crate::prelude::*;

#[derive(Resource)]
pub struct MessageQueue<T>(Vec<T>);

impl<T> MessageQueue<T> {
    /// Record a new message
    pub fn push(&mut self, message: T) {
        self.0.push(message);
    }

    /// Retrieve and clear all logged messages
    pub fn drain(&mut self) -> Vec<T> {
        std::mem::take(&mut self.0)
    }
}

impl<T> Default for MessageQueue<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

pub type EventQueue = MessageQueue<SimEvent>;
pub type ResponseQueue = MessageQueue<SimResponse>;
