use dishrupt_core::EntityId;
use dishrupt_godot::display::*;
use godot::{classes::Label, obj::Gd};

#[allow(unused)]
pub struct DishController {
    entity: EntityId,

    root: GdNode2D,
    price_label: Gd<Label>,
}

impl DishController {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        let price_label = node.get_node_as("Price/Label");

        Self {
            entity,

            price_label,
            root: node,
        }
    }

    pub fn set_price(&mut self, price_str: &str) {
        self.price_label.set_text(price_str);
    }
}
