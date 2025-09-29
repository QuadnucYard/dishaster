use std::num::NonZero;

use dishaster_navigation::NavigationGrid;
use dishrupt_core::EntityId;

use crate::{components::*, prelude::*, resources::*, sim::Simulation};

/// Feature gates controlling which debug payloads are exported.
#[derive(Debug, Clone, Copy)]
pub struct DebugFeatureFlags {
    /// Include per-agent movement debug data.
    pub movement: bool,
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
            nav_grid: true,
            crowd_field: true,
        }
    }

    /// Disable all debug features.
    pub const fn none() -> Self {
        Self {
            movement: false,
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
        if !self.debug_flags.nav_grid {
            return None;
        }

        let mut movement_query = self.world.query::<(Entity, &Movement)>();
        Some(
            movement_query
                .iter(&self.world)
                .map(|(entity, movement)| MovementDebugSnapshot {
                    core_id: EntityId(NonZero::new(entity.to_bits()).unwrap()),
                    position: movement.pos,
                    velocity: movement.velocity,
                    path: movement.path.waypoints.clone(),
                })
                .collect(),
        )
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
