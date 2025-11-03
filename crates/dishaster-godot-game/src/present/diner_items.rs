use dishaster_interface::event::DinerItemsChange;
use dishrupt_core::EntityId;
use dishrupt_godot::display::*;
use godot::{classes::Node2D, prelude::*};

pub struct DinerItemsPresenter {
    entity: EntityId,

    // Slot nodes for item attachment
    tray_slot: Option<Gd<Node2D>>,
    chopsticks_slot: Option<Gd<Node2D>>,

    // Current item state
    tray_entity: Option<EntityId>,
    chopsticks_entity: Option<EntityId>,
    dish_entity: Option<EntityId>,
    is_eating: bool,
}

impl DinerItemsPresenter {
    pub fn new(entity: EntityId, node: GdNode2D) -> Self {
        // Try to get slot nodes (attachment points for items)
        let tray_slot = node.try_get_node_as::<Node2D>("TraySlot");
        let chopsticks_slot = node.try_get_node_as::<Node2D>("ChopsticksSlot");

        Self {
            entity,

            tray_slot,
            chopsticks_slot,

            tray_entity: None,
            chopsticks_entity: None,
            dish_entity: None,
            is_eating: false,
        }
    }

    /// Handle incremental changes to diner items
    pub fn handle_item_change(&mut self, change: DinerItemsChange, stage: &mut Stage) {
        match change {
            DinerItemsChange::PickTray(tray_entity) => {
                self.attach_tray(tray_entity, stage);
                self.reposition_chopsticks_on_tray(stage);
            }
            DinerItemsChange::PickChopsticks(chopsticks_entity) => {
                self.attach_chopsticks(chopsticks_entity, stage);
            }
            DinerItemsChange::PickDish(dish_entity) => {
                self.attach_dish(dish_entity, stage);
            }
            DinerItemsChange::StartEating => {
                self.is_eating = true;
            }
            DinerItemsChange::FinishEating => {
                self.is_eating = false;
            }
            DinerItemsChange::DropAll => {
                self.tray_entity = None;
                self.chopsticks_entity = None;
                self.dish_entity = None;
            }
        }
    }

    fn attach_tray(&mut self, tray_entity: EntityId, stage: &mut Stage) {
        self.tray_entity = Some(tray_entity);

        if let Some(tray_node) = stage.get_godot_node_mut(tray_entity)
            && !self.is_eating
        {
            if let Some(slot) = &self.tray_slot {
                tray_node.reparent(slot);
                tray_node.set_position(Vector2::ZERO);
                tray_node.set_visible(true);
            } else {
                godot_warn!(
                    "Agent {:?} missing tray slot for tray {:?}",
                    self.entity,
                    tray_entity
                );
            }
        }
    }

    fn attach_chopsticks(&mut self, chopsticks_entity: EntityId, stage: &Stage) {
        self.chopsticks_entity = Some(chopsticks_entity);

        if let Some(chopsticks_node) = stage.get_godot_node(chopsticks_entity)
            && !self.is_eating
        {
            let mut chopsticks_node = chopsticks_node.clone();

            // Attach to tray if it exists, otherwise to chopsticks slot
            if let Some(tray_id) = self.tray_entity
                && let Some(tray_node) = stage.get_godot_node(tray_id)
            {
                chopsticks_node.reparent(&**tray_node);
                chopsticks_node.set_position(Vector2::new(0.0, -5.0));
            } else if let Some(slot) = &self.chopsticks_slot {
                chopsticks_node.reparent(slot);
                chopsticks_node.set_position(Vector2::ZERO);
            } else {
                godot_warn!(
                    "Agent {:?} missing chopsticks slot for chopsticks {:?}",
                    self.entity,
                    chopsticks_entity
                );
            }
            chopsticks_node.set_visible(true);
        }
    }

    fn reposition_chopsticks_on_tray(&mut self, stage: &Stage) {
        // When tray is picked up, reposition chopsticks if they already exist
        if let Some(chopsticks_entity) = self.chopsticks_entity
            && let Some(tray_entity) = self.tray_entity
            && let Some(chopsticks_node) = stage.get_godot_node(chopsticks_entity)
            && let Some(tray_node) = stage.get_godot_node(tray_entity)
        {
            let mut chopsticks_node = chopsticks_node.clone();
            chopsticks_node.reparent(&**tray_node);
            chopsticks_node.set_position(Vector2::new(0.0, -5.0));
        }
    }

    fn attach_dish(&mut self, dish_entity: EntityId, stage: &Stage) {
        self.dish_entity = Some(dish_entity);

        if let Some(dish_node) = stage.get_godot_node(dish_entity)
            && !self.is_eating
        {
            let mut dish_node = dish_node.clone();

            // Dish always goes on tray
            if let Some(tray_entity) = self.tray_entity
                && let Some(tray_node) = stage.get_godot_node(tray_entity)
            {
                dish_node.reparent(&**tray_node);
                dish_node.set_position(Vector2::new(0.0, 5.0));
                dish_node.set_visible(true);
            } else {
                godot_warn!(
                    "Agent {:?} has dish {:?} but no tray",
                    self.entity,
                    dish_entity
                );
            }
        }
    }
}
