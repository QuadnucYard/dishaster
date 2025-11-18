//! Localized UI elements for Godot Engine.

#![allow(missing_docs)]

use godot::prelude::*;

/// A button with a localization key.
#[derive(GodotClass)]
#[class(base=Button, init)]
pub struct LocalizedButton {
    /// The message id to use for this Button's text.
    #[export]
    pub message_id: GString,
}

/// A label with a localization key.
#[derive(GodotClass)]
#[class(base=Label, init)]
pub struct LocalizedLabel {
    /// The message id to use for this Label's text.
    #[export]
    pub message_id: GString,
}

/// A label with a localization key.
#[derive(GodotClass)]
#[class(base=RichTextLabel, init)]
pub struct LocalizedRichLabel {
    /// The message id to use for this Label's text.
    #[export]
    pub message_id: GString,
}

/// A label with a localization key.
#[derive(GodotClass)]
#[class(base=LineEdit, init)]
pub struct LocalizedLineEdit {
    /// The message id to use for this LineEdit's placeholder text.
    #[export]
    pub placeholder_message_id: GString,
}

/// A marker for localized tooltips.
#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct LocalizedTooltip {
    /// The message id to use for its parent's tooltip text.
    #[export]
    pub message_id: GString,
}
