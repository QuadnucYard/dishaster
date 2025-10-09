mod collision_overlay;
mod crowd_overlay;
mod distance_overlay;
mod movement_overlay;
mod queue_overlay;

use dishaster_core::snapshots::Snapshot;
use dishrupt_godot::display::DisplayContext2D;
use godot::{classes::Node2D, prelude::*};

use self::{
    collision_overlay::*, crowd_overlay::*, distance_overlay::*, movement_overlay::*,
    queue_overlay::*,
};

pub struct DbgViz {
    pub collision_overlay: CollisionDebugOverlay,
    pub distance_overlay: DistanceDebugOverlay,
    pub crowd_overlay: CrowdDebugOverlay,
    pub movement_overlay: MovementDebugOverlay,
    pub queue_overlay: QueueDebugOverlay,
}

impl DbgViz {
    /// TODO: better pass origin
    pub fn new(root: &Gd<Node2D>, origin: Vector2) -> Self {
        let mut collision_root = root.get_node_as::<Node2D>("%CollisionDebugOverlay");
        collision_root.set_position(origin);
        let collision_overlay = CollisionDebugOverlay::new(collision_root);

        let mut distance_root = root.get_node_as::<Node2D>("%DistanceDebugOverlay");
        distance_root.set_position(origin);
        let distance_overlay = DistanceDebugOverlay::new(distance_root);

        let mut crowd_root = root.get_node_as::<Node2D>("%CrowdDebugOverlay");
        crowd_root.set_position(origin);
        let crowd_overlay = CrowdDebugOverlay::new(crowd_root);

        let mut movement_root = root.get_node_as::<Node2D>("%MovementDebugOverlay");
        movement_root.set_position(origin);
        let movement_overlay = MovementDebugOverlay::new(movement_root);

        let mut queue_root = root.get_node_as::<Node2D>("%QueueDebugOverlay");
        queue_root.set_position(origin);
        let queue_overlay = QueueDebugOverlay::new(queue_root);

        Self {
            collision_overlay,
            distance_overlay,
            crowd_overlay,
            movement_overlay,
            queue_overlay,
        }
    }

    pub fn update(&mut self, snapshot: &Snapshot, display_ctx: &DisplayContext2D) {
        self.collision_overlay
            .present(snapshot.collision_debug.as_ref(), display_ctx);
        self.crowd_overlay
            .present(snapshot.crowd_debug.as_ref(), display_ctx);
        self.movement_overlay
            .present(snapshot.movement_debug.as_ref(), display_ctx);
        self.queue_overlay
            .present(snapshot.queue_debug.as_ref(), display_ctx);
    }
}
