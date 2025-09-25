//! Pathfinding utilities using the `pathfinding` crate.

use ::pathfinding::prelude::astar;

use crate::{prelude::*, *};

/// A request to find a path between two points.
pub struct PathRequest<'a> {
    /// The starting point of the path.
    pub start: Vec2,
    /// The ending point of the path.
    pub end: Vec2,
    /// The collision grid to use for pathfinding.
    pub grid: &'a CollisionGrid,
    /// Inclusive world bounds: [0,width] x [0,height]
    pub world_width: f32,
    /// Inclusive world bounds height
    pub world_height: f32,
    /// Optional crowd cost field to bias path away from agents
    pub crowd: Option<&'a CrowdCostField>,
}

/// Finds a path from a start to an end point using A*.
/// The grid is used to determine walkable tiles.
pub fn find_path(request: PathRequest) -> Option<Vec<Vec2>> {
    let start_tile = request.grid.world_to_grid(request.start);
    let end_tile = request.grid.world_to_grid(request.end);

    // Custom neighbor generator: allow entering the goal tile even if currently occupied.
    // This avoids unbounded search when the end cell is temporarily blocked by another diner.
    let neighbor_fn = |p: &IVec2| {
        let mut neighbors = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let n = *p + IVec2::new(dx, dy);
                // Bounds check: convert to world and ensure within [0,width]x[0,height]
                let world = request.grid.tile_to_world(n);
                if world.x < 0.0
                    || world.y < 0.0
                    || world.x > request.world_width
                    || world.y > request.world_height
                {
                    continue;
                }
                // Check occupancy: allow the goal tile even if occupied, but block others.
                if n == end_tile || !request.grid.is_occupied(n) {
                    neighbors.push((n, 1));
                }
            }
        }
        neighbors
    };

    let result = astar(
        &start_tile,
        |p| {
            let neighbors = neighbor_fn(p);
            neighbors
                .into_iter()
                .map(|(n, base)| {
                    let mut cost = base;
                    if let Some(field) = request.crowd {
                        let extra = field.sample(n);
                        cost += (extra.ceil() as i32).max(0);
                    }
                    (n, cost)
                })
                .collect::<Vec<_>>()
        },
        |&p| (p.x - end_tile.x).abs() + (p.y - end_tile.y).abs(),
        |&p| p == end_tile,
    );

    result.map(|(path, _cost)| {
        path.into_iter()
            .map(|tile_pos| request.grid.tile_to_world(tile_pos))
            .collect()
    })
}
