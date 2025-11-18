use godot::{classes::*, prelude::*};

/// Panic overlay that displays panic information
pub struct PanicOverlay {
    root: Gd<CanvasLayer>,
}

impl PanicOverlay {
    pub fn new(mut root: Gd<CanvasLayer>) -> Self {
        root.set_visible(false);
        Self { root }
    }

    /// Show the panic overlay with the given message
    pub fn show_panic(&mut self, message: &str) {
        if let Some(mut label) = self.root.try_get_node_as::<Label>("%MessageLabel") {
            label.set_text(message);
        }
        self.root.set_visible(true);
    }
}
