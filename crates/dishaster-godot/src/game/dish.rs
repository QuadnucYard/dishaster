use dishaster_core::{models::PricingMethod, snapshots::DishViewModel};
use dishrupt_core::EntityId;
use dishrupt_godot::{display::*, ext::NodeExt};
use godot::{
    classes::{Area2D, Label},
    prelude::*,
};

use crate::game::input::Pickable;

#[allow(unused)]
pub struct DishController {
    entity: EntityId,

    root: GdNode2D,
    area: Gd<Area2D>,
    price_label: Gd<Label>,

    view_model: Option<DishViewModel>,
    original_price: Option<PricingMethod>,
}

impl DishController {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        let area = node.get_child_of_type().unwrap();
        let price_label = node.get_node_as("Price/Label");

        Self {
            entity,

            area,
            price_label,
            root: node,

            view_model: None,
            original_price: None,
        }
    }

    pub fn set_view_model(&mut self, vm: DishViewModel) {
        self.set_price(&match vm.pricing {
            PricingMethod::PerPortion(val) => format!("${:.1}", val),
            PricingMethod::ByWeight(val) => format!("${:.1}", val),
        });
        self.original_price = Some(vm.pricing);
        self.view_model = Some(vm);
    }

    pub fn set_price(&mut self, price_str: &str) {
        self.price_label.set_text(price_str);
    }
}

impl Pickable for DishController {
    fn collider_instance_id(&self) -> InstanceId {
        self.area.instance_id_unchecked()
    }
}
