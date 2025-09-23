use derive_more::*;
use dishrupt_core::{EntityId, display::DisplaySnapshot, prelude::*};
use glam::Vec3Swizzles;
use godot::prelude::*;

use super::context::DisplayContext2D;
use crate::bind::IntoGodot;

#[derive(Deref, DerefMut)]
pub struct GdNode2D(Gd<Node2D>);

impl GdNode2D {
    pub fn new(node: Gd<impl Inherits<Node2D>>) -> Self {
        Self(node.upcast())
    }
}

impl GdNode2D {
    pub fn destroy(&mut self) {}
}

// SAFETY: The world is always used in the main thread.
unsafe impl Send for GdNode2D {}
unsafe impl Sync for GdNode2D {}

/// Pooled handler of Godot Node2D
#[derive(Component)]
pub struct GodotDisplayNode2D {
    pub bind: Option<EntityId>,

    pub node: GdNode2D,
}

impl GodotDisplayNode2D {
    pub fn new(node: GdNode2D) -> Self {
        Self {
            bind: Default::default(),
            node,
        }
    }

    pub fn new_bind(node: GdNode2D, entity: EntityId) -> Self {
        Self {
            bind: Some(entity),
            node,
        }
    }

    pub fn bind_to(&mut self, entity: EntityId) {
        self.bind = Some(entity);
    }

    /*
       pub fn take_reparent(&self) -> Option<Rc<DisplayNode>> {
           self.0.borrow().bind.upgrade().and_then(|display_rc| {
               display_rc
                   .borrow()
                   .reparent_signal
                   .take()
                   .and_then(|t| t.upgrade())
           })
       }
    */

    pub fn set_parent(&mut self, parent: &mut Self) {
        if self.node.get_parent().is_some() {
            self.node.reparent(&*parent.node);
        } else {
            parent.node.add_child(&*self.node);
        }
    }

    pub fn detach(&mut self) {
        let node = &self.node;
        if node.is_instance_valid()
            && let Some(mut parent_node) = node.get_parent()
        {
            parent_node.remove_child(&**node);
        }
    }

    pub fn reset(&mut self) {
        self.node.request_ready();
    }

    pub fn is_valid(&self) -> bool {
        self.node.is_instance_valid() && self.node.get_parent().is_some()
    }

    pub fn destroy(&mut self) {
        self.detach();
    }
}

impl Drop for GodotDisplayNode2D {
    fn drop(&mut self) {
        // need to check scene tree valid
        if self.node.is_instance_valid() {
            // it seems that the instance is always invalid
            // node.queue_free();
            if !self.node.is_queued_for_deletion() {
                self.node.clone().free();
            }
        }
    }
}

pub fn update_godot_display_node2d(
    node_handle: &mut GodotDisplayNode2D,
    snapshot: &DisplaySnapshot,
    ctx: &DisplayContext2D,
) {
    let node = &mut node_handle.node;
    let transform = &snapshot.transform;
    let pos = transform.position * ctx.view_scale;
    node.set_position(Vec2::new(pos.x, pos.y - pos.z).into_godot());
    // node.set_z_index((pos.z * 1e2) as i32);
    node.set_rotation(transform.rotation);
    node.set_scale(transform.scale.xy().into_godot());
}
