use dishaster_channel::events::DishViewModel;
use dishaster_godot_ui::{DishPricePopup, DishPriceView};
use dishaster_models::PricingMethod;
use dishrupt_core::EntityId;
use dishrupt_godot::{display::*, ext::NodeExt, input::event::MouseButtonEvent};
use dishrupt_godot_scene::SceneContext;
use dishrupt_godot_ui::*;
use dishrupt_l10n::tr;
use godot::{
    classes::{Area2D, Label},
    prelude::*,
};

use crate::input::Pickable;

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

impl Pickable for DishController {
    fn collider_instance_id(&self) -> InstanceId {
        self.area.instance_id_unchecked()
    }

    fn on_click(&mut self, ctx: &mut SceneContext, event: &MouseButtonEvent) {
        if !event.pressed
            && let popup = ctx.gui.get_mut::<DishPricePopup>()
            && popup.enabled
            && let (Some(vm), Some(orig_price)) = (&self.view_model, &self.original_price)
        {
            popup.set_view(DishPriceView {
                entity: vm.entity,
                dish_name: tr!("dish-{}.name", vm.dish_id),
                original_price: *orig_price,
                current_price: vm.pricing,
            });
            popup.show();
        }
    }
}
