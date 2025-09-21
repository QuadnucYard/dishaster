use godot::{classes::Node, obj::Gd};

use crate::Gui;

pub struct UIRoot {
    gd: Gd<Node>,
}

impl UIRoot {
    pub fn new(gd: Gd<Node>) -> Self {
        Self { gd }
    }

    pub fn gd(&self) -> Gd<Node> {
        self.gd.clone()
    }

    pub fn add_gui(&self, tree: &dyn Gui) {
        let tree_gd = tree.gd();
        if tree_gd.get_parent().is_none() {
            self.gd.clone().add_child(tree_gd);
        }
    }

    pub fn remove_tree(&self, tree: &dyn Gui) {
        let tree_gd = tree.gd();
        if let Some(mut parent) = tree_gd.get_parent() {
            parent.remove_child(tree_gd);
        }
    }
}
