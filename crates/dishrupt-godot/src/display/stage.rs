use std::cell::OnceCell;

use dishrupt_core::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use slab::Slab;

use super::{DisplayFactory, GodotDisplayNode2D, context::DisplayContext2D};
use crate::display::node::{GdNode2D, update_godot_display_node2d};

type NodeHandle = usize;

pub struct Stage {
    factory: DisplayFactory,

    display_ctx: DisplayContext2D,
    display_world: Slab<GodotDisplayNode2D>,
    /// Map from display entity to Godot node
    core_to_view: FxHashMap<EntityId, NodeHandle>,
    // persistent_nodes: FxHashMap<EntityId, GdNode2D>,
    display_root: OnceCell<EntityId>,

    active: bool,
}

impl Stage {
    pub fn new(display_ctx: DisplayContext2D) -> Self {
        Self {
            factory: Default::default(),
            display_ctx,
            display_world: Default::default(),
            core_to_view: Default::default(),
            display_root: Default::default(),
            active: false,
        }
    }

    pub fn display_context(&self) -> &DisplayContext2D {
        &self.display_ctx
    }

    pub fn set_root(&mut self, display_id: EntityId, gd_node: GdNode2D) {
        godot::global::godot_print!("Stage display root mounted at {}", gd_node.get_path());
        let root_entity = self
            .display_world
            .insert(GodotDisplayNode2D::new_bind(gd_node, display_id));
        self.display_root
            .set(display_id)
            .expect("set display root node");
        self.core_to_view.insert(display_id, root_entity);
        self.active = true;
        self.factory.init();
    }

    pub fn get_godot_node(&self, entity: EntityId) -> Option<&GdNode2D> {
        let e = *self.core_to_view.get(&entity)?;
        let gd_node = self.display_world.get(e)?;
        Some(&gd_node.node)
    }

    pub fn get_godot_node_mut(&mut self, entity: EntityId) -> Option<&mut GdNode2D> {
        let e = *self.core_to_view.get(&entity)?;
        let gd_node = self.display_world.get_mut(e)?;
        Some(&mut gd_node.node)
    }

    pub fn update(&mut self, elapsed_time: f64) {
        if !self.active {
            return;
        }
        self.factory.tidy((elapsed_time * 60.0) as u32);
    }

    pub fn present_direct(&mut self, display: &DisplaySnapshot, gd_node: GdNode2D) -> NodeHandle {
        let node = GodotDisplayNode2D::new_bind(gd_node, display.core_id);
        let e = self.display_world.insert(node);
        self.core_to_view.insert(display.core_id, e);
        // no update performed
        e
    }

    pub fn present<'a>(&mut self, displays: impl Iterator<Item = &'a DisplaySnapshot>) {
        let ctx = &self.display_ctx;

        let mut seen = FxHashSet::default();
        let root = *self.display_root.get().expect("stage root not set");
        seen.insert(root); // always keep root

        let mut reparents = Vec::new();

        // Update existing nodes and create new nodes
        for display in displays {
            seen.insert(display.core_id);

            if let Some(e) = self.core_to_view.get(&display.core_id) {
                // update existing node
                let node = self.display_world.get_mut(*e).unwrap();
                update_godot_display_node2d(node, display, ctx);
            } else {
                // create new node
                let mut gd_node = self.factory.create(&display.proto);
                if let Some(name) = &display.name {
                    gd_node.set_name(name.as_str());
                }
                let mut node = GodotDisplayNode2D::new_bind(gd_node, display.core_id);
                update_godot_display_node2d(&mut node, display, ctx);
                let e = self.display_world.insert(node);
                self.core_to_view.insert(display.core_id, e);

                // currently do not support reparenting.
                reparents.push((display.core_id, display.transform.parent.unwrap_or(root)));
            }
        }

        // Remove invalid nodes
        self.core_to_view.retain(|core_id, &mut e| {
            if seen.contains(core_id) {
                return true;
            }
            if let Some(node) = self.display_world.get_mut(e) {
                node.destroy();
            }
            self.display_world.remove(e);
            false
        });

        // process parent-setting
        for (child, parent) in reparents {
            let (Some(&child_entity), Some(&parent_entity)) = (
                self.core_to_view.get(&child),
                self.core_to_view.get(&parent),
            ) else {
                continue;
            };

            let mut child_node = self
                .display_world
                .get_mut(child_entity)
                .unwrap()
                .node
                .clone();
            let mut parent_node = self
                .display_world
                .get_mut(parent_entity)
                .unwrap()
                .node
                .clone();
            if child_node.get_parent().is_some() {
                child_node.reparent(&*parent_node);
            } else {
                parent_node.add_child(&*child_node);
            }
        }
    }

    pub fn flush(&mut self) {
        self.factory.flush();
    }
}
