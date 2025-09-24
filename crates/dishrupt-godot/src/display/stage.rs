use std::cell::OnceCell;

use dishrupt_core::{EntityId, display::*, prelude::*};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{DisplayFactory, GodotDisplayNode2D, context::DisplayContext2D};
use crate::display::node::{GdNode2D, update_godot_display_node2d};

#[derive(Default)]
pub struct Stage {
    factory: DisplayFactory,

    display_world: World,
    /// Map from display entity to Godot node
    core_to_view: FxHashMap<EntityId, Entity>,
    // persistent_nodes: FxHashMap<EntityId, GdNode2D>,
    display_root: OnceCell<EntityId>,

    active: bool,
}

impl Stage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_root(&mut self, display_id: EntityId, gd_node: GdNode2D) {
        godot::global::godot_print!("Stage display root mounted at {}", gd_node.get_path());
        let root_entity = self
            .display_world
            .spawn(GodotDisplayNode2D::new_bind(gd_node, display_id))
            .id();
        self.display_root
            .set(display_id)
            .expect("set display root node");
        self.core_to_view.insert(display_id, root_entity);
        self.active = true;
        self.factory.init();
    }

    pub fn set_display_context(&mut self, ctx: DisplayContext2D) {
        self.display_world.insert_resource(ctx);
    }
    /*
       pub fn get_godot_node(&self, display: &Rc<DisplayNode>) -> Option<Rc<GodotDisplayNode2D>> {
           self.display_table.get(display).cloned()
       }
    */
    pub fn update(&mut self, elapsed_time: f64) {
        if !self.active {
            return;
        }
        self.factory.tidy((elapsed_time * 60.0) as u32);
    }

    pub fn present_direct(&mut self, display: &DisplaySnapshot, gd_node: GdNode2D) -> Entity {
        let node = GodotDisplayNode2D::new_bind(gd_node, display.core_id);
        let e = self.display_world.spawn(node).id();
        self.core_to_view.insert(display.core_id, e);
        // no update performed
        e
    }

    pub fn present<'a>(&mut self, displays: impl Iterator<Item = &'a DisplaySnapshot>) {
        let ctx = (*self.display_world.resource::<DisplayContext2D>()).clone();

        let mut seen = FxHashSet::default();
        seen.insert(*self.display_root.get().unwrap()); // always keep root

        let mut reparents = Vec::new();

        // Update existing nodes and create new nodes
        for display in displays {
            seen.insert(display.core_id);

            if let Some(e) = self.core_to_view.get(&display.core_id) {
                // update existing node
                let mut entity_ref = self.display_world.entity_mut(*e);
                let mut node = entity_ref.get_mut::<GodotDisplayNode2D>().unwrap();
                update_godot_display_node2d(&mut node, display, &ctx);
            } else {
                // create new node
                let gd_node = self.factory.create(&display.proto);
                let mut node = GodotDisplayNode2D::new_bind(gd_node, display.core_id);
                update_godot_display_node2d(&mut node, display, &ctx);
                let e = self.display_world.spawn(node).id();
                self.core_to_view.insert(display.core_id, e);
            }

            if display.transform.parent.is_modified() {
                reparents.push((
                    display.core_id,
                    display
                        .transform
                        .parent
                        .get()
                        .unwrap_or_else(|| *self.display_root.get().unwrap()),
                ));
            }
        }

        // Remove invalid nodes
        self.core_to_view.retain(|core_id, e| {
            if seen.contains(core_id) {
                return true;
            }
            if let Some(mut node) = self
                .display_world
                .entity_mut(*e)
                .get_mut::<GodotDisplayNode2D>()
            {
                node.destroy();
            }
            self.display_world.despawn(*e);
            false
        });

        // process parent-setting
        for (child, parent) in reparents {
            if let Some(child_entity) = self.core_to_view.get(&child) {
                // reparent
                if let Some(parent_entity) = self.core_to_view.get(&parent) {
                    let mut child_node = self
                        .display_world
                        .get_mut::<GodotDisplayNode2D>(*child_entity)
                        .unwrap()
                        .node
                        .clone();
                    let mut parent_node = self
                        .display_world
                        .get_mut::<GodotDisplayNode2D>(*parent_entity)
                        .unwrap()
                        .node
                        .clone();
                    if child_node.get_parent().is_some() {
                        child_node.reparent(&parent_node);
                    } else {
                        parent_node.add_child(&child_node);
                    }
                }
            }
        }
    }

    pub fn flush(&mut self) {
        self.factory.flush();
    }
}
