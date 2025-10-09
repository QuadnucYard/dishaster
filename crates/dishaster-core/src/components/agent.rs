use dishaster_navigation::NavPath;

use crate::prelude::*;

/// Marker component for identifying agent entities
#[derive(Component)]
pub struct AgentTag;

/// Runtime component for movement state and behavior
#[derive(Component)]
pub struct Movement {
    /// Base walking speed in meters per second.
    pub walking_speed: f32,
    /// Speed factor applied to this entity's base movement speed.
    pub speed_factor: f32,
    /// The radius of the agent for collision avoidance.
    pub radius: f32,
    /// How impatient the agent is (0.0 = patient, 1.0 = very impatient).
    pub impatience: f32,
    /// The amount of responsibility an agent has to avoid other agents.
    pub avoidance_responsibility: f32,

    /// Current position in the canteen
    pub pos: Vec2,
    /// Current velocity vector
    pub velocity: Vec2,
    /// The calculated path to the target_pos.
    pub path: NavPath,
    /// The target position to move towards.
    pub pending_target: Option<Vec2>,
    /// Whether the target has been reached.
    pub target_reached: bool,
    /// The last tick when the path was calculated. It is set when a new path is assigned.
    pub last_path_tick: Tick,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            walking_speed: 1.0,
            speed_factor: 1.0,
            radius: 0.0,
            impatience: 1.0,
            avoidance_responsibility: 1.0,

            pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            path: Default::default(),
            target_reached: false,
            pending_target: None,
            last_path_tick: 0,
        }
    }
}

impl Movement {
    /// Returns true if the agent has a path to follow, or is en route to a target.
    pub fn has_path(&self) -> bool {
        !self.path.is_empty() || self.pending_target.is_some()
    }

    /// Clear the current path and stop movement.
    pub fn stop(&mut self) {
        self.path.clear();
        self.pending_target = None;
        self.target_reached = true;
        self.velocity = Vec2::ZERO;
    }
}

/// Component for entities that own and manage one or more queue lanes.
#[derive(Component)]
pub struct LaneOwner {
    /// Queue lanes managed by this entity.
    pub lanes: Vec<Entity>,
}

/// A queue lane in the canteen.
#[derive(Component)]
pub struct QueueLane {
    /// Anchor position of the lane (x, y), for reference
    pub anchor: Vec2,
    /// Direction vector of the lane (normalized)
    pub direction: Vec2,
}

/// Current members of the queue lane, ordered from front to back.
/// The fields are dynamically updated by the queueing system.
#[derive(Component)]
pub struct QueueLaneMembers {
    /// Current members in the lane, ordered from front (0) to back (n-1).
    pub members: Vec<Entity>,
    /// Position at the rear of the queue, where new members should go.
    /// When the queue is empty, this is around the anchor position.
    pub rear_pos: Vec2,
}

/// Intent marker for diners heading toward a service queue.
#[derive(Component)]
pub struct QueueIntent {
    /// QueueLane entity the diner plans to join.
    pub lane: Entity,
}

impl QueueIntent {
    /// Create a new queue intent for the specified lane at the given time.
    pub fn new(lane: Entity) -> Self {
        Self { lane }
    }
}

/// Marker component for diners who are currently members of a queue.
#[derive(Component)]
pub struct QueueMember {
    /// QueueLane entity the agent is currently in.
    pub lane: Entity,
    /// Current ranking in the queue (0 = front of the line). Maintained by the queueing system.
    pub ranking: usize,
}

impl QueueMember {
    /// Create a new QueueMember for the specified lane.
    pub fn new(lane: Entity) -> Self {
        Self {
            lane,
            ranking: usize::MAX,
        }
    }
}
