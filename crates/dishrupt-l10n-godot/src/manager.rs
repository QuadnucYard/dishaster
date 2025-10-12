use dishrupt_l10n::try_tr_plain;
use godot::{classes::Node, global::push_warning, prelude::*};

use crate::localized::{LocalizedButton, LocalizedLabel};

enum Localizable {
    Label(Gd<LocalizedLabel>),
    Button(Gd<LocalizedButton>),
}

impl Localizable {
    fn node_name(&self) -> StringName {
        match self {
            Localizable::Label(a) => a.get_name(),
            Localizable::Button(a) => a.get_name(),
        }
    }

    fn node_path(&self) -> NodePath {
        match self {
            Localizable::Label(a) => a.get_path(),
            Localizable::Button(a) => a.get_path(),
        }
    }
}

/// Manages localization for a set of localizable items
#[derive(Default)]
pub struct LocalizationManager {
    items: Vec<(Localizable, String)>,
}

impl LocalizationManager {
    /// Create a new, empty manager
    pub fn new() -> Self {
        Default::default()
    }

    /// Collect all localizable items under the given root
    pub fn collect(&mut self, root: Gd<Node>) {
        self.items.clear();

        // Traverse the tree
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if let Ok(label) = node.clone().try_cast::<LocalizedLabel>() {
                let key = &label.bind().message_id;
                if key.is_empty() {
                    push_warning(&[
                        format!("The key for node `{}` is empty!", node.get_path()).to_variant()
                    ]);
                } else {
                    self.items
                        .push((Localizable::Label(label.clone()), key.to_string()));
                }
            }
            if let Ok(button) = node.clone().try_cast::<LocalizedButton>() {
                let key = &button.bind().message_id;
                if key.is_empty() {
                    push_warning(&[
                        format!("The key for node `{}` is empty!", node.get_path()).to_variant()
                    ]);
                } else {
                    self.items
                        .push((Localizable::Button(button.clone()), key.to_string()));
                }
            }
            stack.extend(node.get_children().iter_shared());
        }
    }

    /// Update all collected items
    pub fn update(&mut self) {
        for (loc, key) in self.items.iter_mut() {
            let Some(value) = try_tr_plain(key) else {
                push_warning(&[format!(
                    "Failed to get message `{key}` for node `{}`, node path: {}",
                    loc.node_name(),
                    loc.node_path()
                )
                .to_variant()]);
                continue;
            };
            match loc {
                Localizable::Label(label) => label.set_text(&value),
                Localizable::Button(button) => button.set_text(&value),
            }
        }
    }
}
