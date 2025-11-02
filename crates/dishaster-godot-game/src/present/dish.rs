use dishaster_ui_protocol::UiCommand;
use dishaster_views::{DishPriceView, DishView, PricingMethod};
use dishrupt_core::EntityId;
use dishrupt_godot::{NodeExt, display::*, input::event::MouseButtonEvent};
use dishrupt_l10n::tr;
use godot::{
    classes::{Area2D, Label},
    prelude::*,
};

use crate::input::{Pickable, PickingContext};

#[allow(unused)]
pub struct DishPresenter {
    entity: EntityId,

    root: GdNode2D,
    area: Gd<Area2D>,
    price_label: Gd<Label>,

    view_model: Option<DishView>,
    original_price: Option<PricingMethod>,
}

impl DishPresenter {
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

    pub fn set_view(&mut self, vm: DishView) {
        self.set_price(vm.pricing);
        self.original_price = Some(vm.pricing);
        self.view_model = Some(vm);
    }

    pub fn set_price(&mut self, pricing: PricingMethod) {
        self.price_label.set_text(&match pricing {
            PricingMethod::PerPortion(val) => format!("${:.1}", val),
            PricingMethod::ByWeight(val) => format!("${:.1}", val),
        });
    }
}

impl Pickable for DishPresenter {
    fn collider_instance_id(&self) -> InstanceId {
        self.area.instance_id_unchecked()
    }

    fn on_click(&mut self, ctx: &mut PickingContext, event: &MouseButtonEvent) {
        if !event.pressed
            && let (Some(vm), Some(orig_price)) = (&self.view_model, &self.original_price)
        {
            ctx.cmds.push(UiCommand::OpenDishPriceEditor(DishPriceView {
                entity: vm.entity,
                dish_name: tr!("dish-{}.name", vm.dish_id),
                original_price: *orig_price,
                current_price: vm.pricing,
            }));
        }
    }
}
