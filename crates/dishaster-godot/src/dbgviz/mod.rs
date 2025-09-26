mod collision_overlay;
mod crowd_overlay;
mod movement_overlay;

use dishaster_core::snapshots::Snapshot;
use dishrupt_godot::display::DisplayContext2D;
use godot::{classes::Node2D, prelude::*};

use self::{collision_overlay::*, crowd_overlay::*, movement_overlay::*};

pub struct DbgViz {
    collision_overlay: CollisionDebugOverlay,
    crowd_overlay: CrowdDebugOverlay,
    movement_overlay: MovementDebugOverlay,
}

impl DbgViz {
    /// TODO: better pass origin
    pub fn new(root: &Gd<Node2D>, origin: Vector2) -> Self {
        let mut collision_root = root.get_node_as::<Node2D>("%CollisionDebugOverlay");
        collision_root.set_position(origin);
        let collision_overlay = CollisionDebugOverlay::new(collision_root);

        let mut crowd_root = root.get_node_as::<Node2D>("%CrowdDebugOverlay");
        crowd_root.set_position(origin);
        let crowd_overlay = CrowdDebugOverlay::new(crowd_root);

        let mut movement_root = root.get_node_as::<Node2D>("%MovementDebugOverlay");
        movement_root.set_position(origin);
        let movement_overlay = MovementDebugOverlay::new(movement_root);

        Self {
            collision_overlay,
            crowd_overlay,
            movement_overlay,
        }
    }

    pub fn update(&mut self, snapshot: &Snapshot, display_ctx: &DisplayContext2D) {
        self.collision_overlay
            .present(snapshot.collision_debug.as_ref(), display_ctx);
        self.crowd_overlay
            .present(snapshot.crowd_debug.as_ref(), display_ctx);
        self.movement_overlay
            .present(snapshot.movement_debug.as_ref(), display_ctx);
    }
}
