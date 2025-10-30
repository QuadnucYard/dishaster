use std::any::Any;

/// Type-erased request to the GUI system.
pub trait UiRequest: Send + Any {}
