use as_any::AsAny;

/// Type-erased request to the GUI system.
pub trait UiRequest: Send + AsAny {}
