use dishaster_interface::event::DinerItemsChange;
use dishrupt_core::EntityId;
use dishrupt_godot_display::{GdNode2D, Stage};
use godot::{classes::Node2D, prelude::*};

pub struct DinerItemsPresenter {
    entity: EntityId,

    // Slot nodes for item attachment
    tray_slot: Option<Gd<Node2D>>,
    chopsticks_slot: Option<Gd<Node2D>>,

    // Current item state
    tray_entity: Option<EntityId>,
    chopsticks_entity: Option<EntityId>,
    dish_entities: Vec<EntityId>,
    is_eating: bool,
    eating_time: f32,

    anim_speed: f32,
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
            dish_entities: Default::default(),
            is_eating: false,
            eating_time: 0.0,

            anim_speed: godot::global::randf_range(5.0, 10.0) as f32,
        }
    }

    pub fn process(&mut self, delta: f64, stage: &mut Stage) {
        // Animate the chopsticks if the diner is eating
        if self.is_eating {
            self.animate_chopsticks(stage);
            self.eating_time += delta as f32;
        }
    }

    fn animate_chopsticks(&mut self, stage: &mut Stage) {
        if let Some(chopsticks_entity) = self.chopsticks_entity
            && let Some(chopsticks_node) = stage.get_godot_node_mut(chopsticks_entity)
        {
            let anim_speed = self.anim_speed;
            let anim_amplitude = 20.0;
            let y_offset = anim_amplitude * ((self.eating_time * anim_speed).sin() - 3.0);
            chopsticks_node.set_position(Vector2::new(10.0, y_offset));
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
            DinerItemsChange::StartEating(table_entity, seat_index) => {
                self.is_eating = true;
                self.put_tray_on_table(table_entity, seat_index, stage);
            }
            DinerItemsChange::FinishEating => {
                self.is_eating = false;
                self.take_tray_from_table(stage);
            }
            DinerItemsChange::DropAll => {
                self.tray_entity = None;
                self.chopsticks_entity = None;
                self.dish_entities.clear();
            }
        }
    }

    fn attach_tray(&mut self, tray_entity: EntityId, stage: &mut Stage) {
        self.tray_entity = Some(tray_entity);

        if let Some(tray_node) = stage.get_godot_node_mut(tray_entity)
            && let Some(slot) = &self.tray_slot
        {
            tray_node.reparent(slot);
            tray_node.set_position(Vector2::ZERO);
        } else {
            godot_warn!(
                "Agent {:?} missing tray slot for tray {:?}",
                self.entity,
                tray_entity
            );
        }
    }

    fn attach_chopsticks(&mut self, chopsticks_entity: EntityId, stage: &Stage) {
        self.chopsticks_entity = Some(chopsticks_entity);

        if let Some(chopsticks_node) = stage.get_godot_node(chopsticks_entity) {
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
        }
    }

    fn reposition_chopsticks_on_tray(&mut self, stage: &mut Stage) {
        // When tray is picked up, reposition chopsticks if they already exist
        if let Some(chopsticks_entity) = self.chopsticks_entity
            && let Some(tray_entity) = self.tray_entity
            && let Some((chopsticks_node, tray_node)) =
                stage.get_godot_node2_mut(chopsticks_entity, tray_entity)
        {
            chopsticks_node.reparent(&**tray_node);
            chopsticks_node.set_position(Vector2::new(0.0, -5.0));
        }
    }

    fn attach_dish(&mut self, dish_entity: EntityId, stage: &mut Stage) {
        self.dish_entities.push(dish_entity);

        if let Some(tray_entity) = self.tray_entity
            && let Some((dish_node, tray_node)) =
                stage.get_godot_node2_mut(dish_entity, tray_entity)
        {
            // Dish always goes on tray
            dish_node.reparent(&**tray_node);
            let offset = 5.0 * (self.dish_entities.len() - 1) as f32; // Stack dishes with offset
            dish_node.set_position(Vector2::new(0.0, 5.0 - offset));
        } else {
            godot_warn!(
                "Agent {:?} has dish {:?} but no tray",
                self.entity,
                dish_entity
            );
        }
    }

    fn put_tray_on_table(&mut self, table_entity: EntityId, seat_index: usize, stage: &mut Stage) {
        if let Some(tray_entity) = self.tray_entity
            && let Some((tray_node, table_node)) =
                stage.get_godot_node2_mut(tray_entity, table_entity)
            && let Some(slot_container) = table_node.get_node_or_null("DishSlots")
            && let Some(slot_node) = slot_container.get_child(seat_index as i32)
        {
            tray_node.reparent(&slot_node);
            tray_node.set_position(Vector2::ZERO);
        }
    }

    fn take_tray_from_table(&mut self, stage: &mut Stage) {
        if let Some(tray_entity) = self.tray_entity
            && let Some(tray_node) = stage.get_godot_node_mut(tray_entity)
            && let Some(slot) = &self.tray_slot
        {
            tray_node.reparent(slot);
            tray_node.set_position(Vector2::ZERO);
        }
    }
}
