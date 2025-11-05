use dishaster_ui_protocol::UiCommand;
use dishrupt_core::EntityId;
use dishrupt_godot::{display::*, input::event::MouseButtonEvent};
use godot::{classes::Area2D, prelude::*};

use crate::input::{Pickable, PickingContext};

pub struct DispenserPresenter {
    entity: EntityId,
    #[allow(unused)]
    root: GdNode2D,
    area: Gd<Area2D>,
}

impl DispenserPresenter {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        // Get the Area2D child for collision detection
        let area = node
            .try_get_node_as::<Area2D>("Area2D")
            .expect("Dispenser prefab must have Area2D child");

        Self {
            entity,
            root: node,
            area,
        }
    }

    #[allow(unused)]
    pub fn set_stock(&mut self, current: u32, capacity: u32) {
        // TODO: Update visual indicator based on stock level
        // For now, we just store the entity and make it clickable
    }
}

impl Pickable for DispenserPresenter {
    fn collider_instance_id(&self) -> InstanceId {
        self.area.instance_id_unchecked()
    }

    fn on_click(&mut self, ctx: &mut PickingContext, _event: &MouseButtonEvent) {
        godot_print!("Dispenser clicked: {:?}", self.entity);
        // Send command to request refill
        ctx.cmds.push(UiCommand::RefillDispenser(self.entity));
    }
}
