mod collision_overlay;
mod crowd_overlay;
mod distance_overlay;
mod movement_overlay;
mod queue_overlay;

use dishaster_interface::snapshots::DebugSnapshots;
use dishrupt_godot::{NodeExt, display::DisplayContext2D};
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
    /// Creates a new debug visualization manager attached to the given root node.
    pub fn new(mut root: Gd<Node2D>) -> Self {
        let collision_overlay =
            CollisionDebugOverlay::new(root.add_child_as("CollisionDebugOverlay"));
        let distance_overlay = DistanceDebugOverlay::new(root.add_child_as("DistanceDebugOverlay"));
        let crowd_overlay = CrowdDebugOverlay::new(root.add_child_as("CrowdDebugOverlay"));
        let movement_overlay = MovementDebugOverlay::new(root.add_child_as("MovementDebugOverlay"));
        let queue_overlay = QueueDebugOverlay::new(root.add_child_as("QueueDebugOverlay"));

        Self {
            collision_overlay,
            distance_overlay,
            crowd_overlay,
            movement_overlay,
            queue_overlay,
        }
    }

    pub fn update(&mut self, snapshot: &DebugSnapshots, display_ctx: &DisplayContext2D) {
        self.collision_overlay
            .present(snapshot.collision.as_ref(), display_ctx);
        self.crowd_overlay
            .present(snapshot.crowd.as_ref(), display_ctx);
        self.movement_overlay
            .present(snapshot.movement.as_ref(), display_ctx);
        self.queue_overlay
            .present(snapshot.queues.as_ref(), display_ctx);
    }
}
