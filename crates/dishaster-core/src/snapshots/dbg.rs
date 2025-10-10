use dishaster_navigation::NavigationGrid;
use dishrupt_core::EntityId;
use rustc_hash::FxHashMap;

use crate::{components::*, prelude::*, sim::Simulation};

/// Feature gates controlling which debug payloads are exported.
#[derive(Debug, Clone, Copy)]
pub struct DebugFeatureFlags {
    /// Include per-agent movement debug data.
    pub movement: bool,
    /// Include queue lane debug data.
    pub queues: bool,
    /// Include collision grid occupancy visualization data.
    pub nav_grid: bool,
    /// Include crowd cost field visualization data.
    pub crowd_field: bool,
}

impl DebugFeatureFlags {
    /// Enable all debug features.
    pub const fn all() -> Self {
        Self {
            movement: true,
            queues: true,
            nav_grid: true,
            crowd_field: true,
        }
    }

    /// Disable all debug features.
    pub const fn none() -> Self {
        Self {
            movement: false,
            queues: false,
            nav_grid: false,
            crowd_field: false,
        }
    }
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

impl Simulation {
    pub(crate) fn snapshot_movement(&mut self) -> Option<Vec<MovementDebugSnapshot>> {
        if !self.debug_flags.movement {
            return None;
        }

        let mut movement_query = self.world.query::<(Entity, &Movement)>();
        Some(
            movement_query
                .iter(&self.world)
                .map(|(entity, movement)| MovementDebugSnapshot {
                    core_id: entity.into(),
                    position: movement.pos,
                    velocity: movement.velocity,
                    path: movement.path.waypoints.clone(),
                })
                .collect(),
        )
    }

    pub(crate) fn snapshot_queue(&mut self) -> Option<Vec<QueueLaneDebugSnapshot>> {
        if !self.debug_flags.queues {
            return None;
        }

        let mut intents_by_lane: FxHashMap<Entity, Vec<_>> = FxHashMap::default();
        let mut intent_query = self.world.query::<(Entity, &QueueIntent, &Movement)>();
        for (entity, intent, movement) in intent_query.iter(&self.world) {
            intents_by_lane
                .entry(intent.lane)
                .or_default()
                .push(QueueIntentDebugSnapshot {
                    core_id: entity.into(),
                    position: movement.pos,
                });
        }

        let mut lane_query = self
            .world
            .query::<(Entity, &QueueLane, &QueueLaneMembers)>();
        let lanes = lane_query
            .iter(&self.world)
            .map(|(lane_entity, lane, members)| {
                let mut member_snapshots = Vec::with_capacity(members.members.len());
                for &member_entity in members.members.iter() {
                    let Some(movement) = self.world.get::<Movement>(member_entity) else {
                        continue;
                    };
                    member_snapshots.push(QueueMemberDebugSnapshot {
                        core_id: member_entity.into(),
                        position: movement.pos,
                    });
                }

                QueueLaneDebugSnapshot {
                    lane_id: lane_entity.into(),
                    anchor: lane.anchor,
                    direction: lane.direction,
                    rear_pos: members.rear_pos,
                    members: member_snapshots,
                    intents: intents_by_lane.remove(&lane_entity).unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();

        if lanes.is_empty() { None } else { Some(lanes) }
    }

    pub(crate) fn snapshot_collision(&mut self) -> Option<CollisionGridDebugSnapshot> {
        None
        // if !self.debug_flags.nav_grid {
        //     return None;
        // }

        // let grid = self.world.resource::<CollisionGridRes>();
        // let cells = grid.debug_cells();
        // if cells.is_empty() {
        //     return None;
        // }

        // let cell_size = grid.cell_size();
        // let size = Vec2::splat(cell_size);
        // Some(CollisionGridDebugSnapshot {
        //     cell_size,
        //     cells: cells
        //         .into_iter()
        //         .map(|(coord, count)| {
        //             let center = grid.tile_to_world(coord);
        //             CollisionCellDebugSnapshot {
        //                 coord,
        //                 center,
        //                 size,
        //                 occupancy: count as u32,
        //             }
        //         })
        //         .collect(),
        // })
    }

    pub(crate) fn snapshot_crowd(&mut self) -> Option<CrowdFieldDebugSnapshot> {
        if !self.debug_flags.crowd_field {
            return None;
        }

        let field = &self.world.resource::<ResWrapper<NavigationGrid>>().crowd;

        let cell_size = field.cell_size();
        let dimensions = field.tile_dimensions();
        let tiles = field.costs().flatten().clone();
        Some(CrowdFieldDebugSnapshot {
            cell_size,
            origin: Default::default(),
            dimensions,
            costs: tiles,
        })
    }
}
