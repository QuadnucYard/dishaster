use derive_more::{Deref, DerefMut};
use dishrupt_core::prelude::*;
use dishrupt_godot_utils::IntoGodot;
use godot::prelude::*;

use super::context::DisplayContext2D;

/// Pooled handler of Godot Node2D
#[derive(Debug, Clone, Deref, DerefMut)]
pub struct GdNode2D(Gd<Node2D>);

impl GdNode2D {
    /// Create a new GdNode2D from a Godot Node2D.
    pub fn new(node: Gd<impl Inherits<Node2D>>) -> Self {
        Self(node.upcast())
    }
}

impl GdNode2D {
    /// Destroy the node.
    pub fn destroy(&mut self) {}
}

// SAFETY: The world is always used in the main thread.
unsafe impl Send for GdNode2D {}
unsafe impl Sync for GdNode2D {}

/// Pooled handler of Godot Node2D
pub struct GodotDisplayNode2D {
    /// Bound entity id
    pub bind: Option<EntityId>,

    /// Godot node
    pub node: GdNode2D,
}

impl GodotDisplayNode2D {
    /// Create a new GodotDisplayNode2D from a Godot Node2D.
    pub fn new(node: GdNode2D) -> Self {
        Self {
            bind: Default::default(),
            node,
        }
    }

    /// Create a new GodotDisplayNode2D bound to an entity.
    pub fn new_bind(node: GdNode2D, entity: EntityId) -> Self {
        Self {
            bind: Some(entity),
            node,
        }
    }

    /// Bind to an entity.
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

    /// Set the parent node.
    pub fn set_parent(&mut self, parent: &mut Self) {
        if self.node.get_parent().is_some() {
            self.node.reparent(&*parent.node);
        } else {
            parent.node.add_child(&*self.node);
        }
    }

    /// Detach from the parent node.
    pub fn detach(&mut self) {
        let node = &self.node;
        if node.is_instance_valid()
            && let Some(mut parent_node) = node.get_parent()
        {
            parent_node.remove_child(&**node);
        }
    }

    /// Reset the node state.
    pub fn reset(&mut self) {
        self.node.request_ready();
    }

    /// Check if the node is valid.
    pub fn is_valid(&self) -> bool {
        self.node.is_instance_valid() && self.node.get_parent().is_some()
    }

    /// Destroy the node.
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
                (*self.node).clone().free();
            }
        }
    }
}

/// Update the Godot Node2D from the display snapshot.
pub fn update_godot_display_node2d(
    node_handle: &mut GodotDisplayNode2D,
    snapshot: &DisplaySnapshot,
    ctx: &DisplayContext2D,
) {
    let node = &mut node_handle.node;
    let transform = &snapshot.transform;
    node.set_position(ctx.to_display_space(transform.position));
    // node.set_z_index((pos.z * 1e2) as i32);
    node.set_rotation(transform.rotation);
    node.set_scale(transform.scale.xy().into_godot());
}
