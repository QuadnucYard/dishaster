//! Simulation state snapshots for rendering and debugging.

use dishrupt_core::prelude::*;

use crate::Tick;

/// Simulation state snapshot for rendering
pub struct Snapshot {
    /// Statistics and metrics for the current simulation state.
    pub stats: DayStats,
    /// Display graph data for rendering the current frame.
    pub display: Vec<DisplaySnapshot>,
    /// Debug visualization snapshots.
    pub debug: DebugSnapshots,
}

/// Statistics and metrics for the current simulation state.
pub struct DayStats {
    /// Simulation timestamp in seconds.
    pub time_seconds: f64,
    /// Total simulation ticks since start.
    pub tick: Tick,

    /// Current number of live diners in the simulation.
    pub live_diners: usize,
    /// Total diner visits since start of day.
    pub total_visits: usize,
}

/// Feature gates controlling which debug payloads are exported.
#[derive(Debug, Clone, Copy)]
pub struct DebugFlags {
    /// Include per-agent movement debug data.
    pub movement: bool,
    /// Include queue lane debug data.
    pub queues: bool,
    /// Include collision grid occupancy visualization data.
    pub nav_grid: bool,
    /// Include crowd cost field visualization data.
    pub crowd_field: bool,
    /// Include diner debug data.
    pub diners: bool,
}

impl DebugFlags {
    /// Enable all debug features.
    pub const fn all() -> Self {
        Self {
            movement: true,
            queues: true,
            nav_grid: true,
            crowd_field: true,
            diners: true,
        }
    }

    /// Disable all debug features.
    pub const fn none() -> Self {
        Self {
            movement: false,
            queues: false,
            nav_grid: false,
            crowd_field: false,
            diners: false,
        }
    }
}

/// Collection of debug visualization snapshots.
pub struct DebugSnapshots {
    /// Per-agent movement debug data collected for visualization.
    pub movement: Option<Vec<MovementDebugSnapshot>>,
    /// Queue lane debug data collected for visualization.
    pub queues: Option<Vec<QueueLaneDebugSnapshot>>,
    /// Collision grid occupancy data when debug is enabled.
    pub collision: Option<CollisionGridDebugSnapshot>,
    /// Crowd cost field data when debug is enabled.
    pub crowd: Option<CrowdFieldDebugSnapshot>,
    /// Diner debug snapshots
    pub diners: Option<Vec<DinerDebugSnapshot>>,
}

/// Debug visualization payload for an agent's movement state.
pub struct MovementDebugSnapshot {
    /// Identifier of the core entity this debug data describes.
    pub core_id: EntityId,
    /// Current simulation-space position of the agent.
    pub position: Vec2,
    /// Current velocity vector in simulation space.
    pub velocity: Vec2,
    /// Remaining waypoints describing the agent's planned path.
    pub path: Vec<Vec2>,
}

/// Debug payload describing a queue lane and its occupants.
pub struct QueueLaneDebugSnapshot {
    /// Identifier of the lane entity itself.
    pub lane_id: EntityId,
    /// Anchor position of the queue lane in simulation space.
    pub anchor: Vec2,
    /// Direction vector pointing from the anchor toward the rear of the queue.
    pub direction: Vec2,
    /// Latest estimated rear position of the queue.
    pub rear_pos: Vec2,
    /// Members currently occupying the queue, ordered from front to back.
    pub members: Vec<QueueMemberDebugSnapshot>,
    /// Agents with intents to join the queue, typically approaching the rear.
    pub intents: Vec<QueueIntentDebugSnapshot>,
}

/// Debug payload describing an individual queue member.
pub struct QueueMemberDebugSnapshot {
    /// Identifier of the agent occupying the queue.
    pub core_id: EntityId,
    /// Current simulation-space position of the agent.
    pub position: Vec2,
}

/// Debug payload describing an active queue intent.
pub struct QueueIntentDebugSnapshot {
    /// Identifier of the agent planning to join the queue.
    pub core_id: EntityId,
    /// Current simulation-space position of the agent.
    pub position: Vec2,
}

/// Debug visualization payload for a single collision grid cell.
pub struct CollisionCellDebugSnapshot {
    /// Grid coordinate of the cell.
    pub coord: IVec2,
    /// World-space center position of the cell.
    pub center: Vec2,
    /// World-space size of the cell.
    pub size: Vec2,
    /// Number of entities occupying the cell.
    pub occupancy: u32,
}

/// Debug visualization payload for the collision grid occupancy data.
pub struct CollisionGridDebugSnapshot {
    /// Size of each collision grid cell in meters.
    pub cell_size: f32,
    /// Populated cells within the collision grid.
    pub cells: Vec<CollisionCellDebugSnapshot>,
}

/// Debug visualization payload for the crowd cost field.
pub struct CrowdFieldDebugSnapshot {
    /// Size of each crowd cost tile in meters.
    pub cell_size: f32,
    /// Minimum tile coordinate covered by the field.
    pub origin: IVec2,
    /// Tile-space dimensions (width, height) of the field.
    pub dimensions: USizeVec2,
    /// Tile costs, arranged in row-major order.
    pub costs: Vec<f32>,
}

/// Debug visualization payload for a diner entity.
pub struct DinerDebugSnapshot {
    /// Identifier of the core entity this debug data describes.
    pub core_id: EntityId,
    /// Current goal state as a string.
    pub goal_str: EcoString,
}
