//! Localized UI elements for Godot Engine.

#![allow(missing_docs)]

use godot::prelude::*;

/// A button with a localization key.
#[derive(GodotClass)]
#[class(base=Button, init)]
pub struct LocalizedButton {
    /// The message id to use for this button's text.
    #[export]
    pub message_id: GString,
}

/// A label with a localization key.
#[derive(GodotClass)]
#[class(base=Label, init)]
pub struct LocalizedLabel {
    /// The message id to use for this label's text.
    #[export]
    pub message_id: GString,
}
