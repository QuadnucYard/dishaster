use dishaster_navigation::{NavPath, PathTarget};

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
    /// This also maps to urgency for speed calculations.
    pub impatience: f32,
    /// The amount of responsibility an agent has to avoid other agents.
    pub avoidance_responsibility: f32,

    // Dynamic speed system fields
    /// Current actual speed after all factors (used with EMA smoothing).
    pub current_speed: f32,
    /// Last time the target speed was updated (for periodic recalculation).
    pub last_speed_update: f32,

    /// Current position in the canteen
    pub pos: Vec2,
    /// Current velocity vector
    pub velocity: Vec2,
    /// The calculated path to the target_pos.
    pub path: NavPath,
    /// The target position to move towards.
    pub pending_target: Option<PathTarget>,
    /// The current target being pursued along the path.
    pub current_target: Option<PathTarget>,
    /// Whether the last requested target has been reached.
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
            current_speed: 1.0, // Initialize to base speed
            last_speed_update: 0.0,

            pos: Vec2::ZERO,
            velocity: Vec2::ZERO,
            path: Default::default(),
            target_reached: false,
            pending_target: None,
            current_target: None,
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
    pub fn stop_as_reached(&mut self) {
        self.path.clear();
        self.pending_target = None;
        self.current_target = None;
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
    /// Entity that owns this lane (e.g., a service window).
    pub owner: Entity,
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

/// Bounded history of recent service completion times for a queue lane.
/// Used to estimate actual wait times based on recent performance.
#[derive(Component)]
pub struct QueueServiceHistory {
    /// Ring buffer of recent service durations (in seconds).
    /// Each entry represents the time between consecutive service completions.
    history: Vec<f32>,
    /// Index where next service time will be written.
    write_index: usize,
    /// Timestamp of the last service completion (simulation time in seconds).
    last_service_time: Option<f64>,
}

impl QueueServiceHistory {
    /// Maximum number of service time samples to keep.
    const HISTORY_SIZE: usize = 10;

    /// Create a new empty service history.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            write_index: 0,
            last_service_time: None,
        }
    }

    /// Record a service completion at the given simulation time.
    pub fn record_service(&mut self, current_time: f64) {
        if let Some(last_time) = self.last_service_time {
            let service_duration = (current_time - last_time) as f32;

            // Only record reasonable service times (1-120 seconds)
            if service_duration > 1.0 && service_duration < 120.0 {
                if self.history.len() < Self::HISTORY_SIZE {
                    self.history.push(service_duration);
                } else {
                    self.history[self.write_index] = service_duration;
                    self.write_index = (self.write_index + 1) % Self::HISTORY_SIZE;
                }
            }
        }

        self.last_service_time = Some(current_time);
    }

    /// Get the average service time per person based on recent history.
    /// Returns None if insufficient data is available.
    pub fn average_service_time(&self) -> Option<f32> {
        if self.history.is_empty() {
            return None;
        }

        let sum: f32 = self.history.iter().sum();
        Some(sum / self.history.len() as f32)
    }

    /// Get the estimated wait time for a given queue position.
    /// Falls back to default estimate if insufficient history.
    /// Blends historical average with default based on sample size for stability.
    /// Uses pessimistic (max of historical and default) to better catch queue issues.
    pub fn estimate_wait_time(&self, queue_position: usize, default_per_person: f32) -> f32 {
        let time_per_person = if self.history.is_empty() {
            // No history yet, use default
            default_per_person
        } else if self.history.len() < Self::HISTORY_SIZE / 2 {
            // Limited history, use pessimistic estimate (take the higher value)
            // This ensures we catch queue problems early
            let avg = self.average_service_time().unwrap();
            avg.max(default_per_person)
        } else {
            // Sufficient history, add 20% buffer for variability and peak times
            self.average_service_time().unwrap() * 1.2
        };

        queue_position as f32 * time_per_person
    }
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
