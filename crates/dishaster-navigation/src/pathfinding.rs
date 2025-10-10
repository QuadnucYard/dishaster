//! Pathfinding utilities using the `pathfinding` crate.

use ::pathfinding::prelude::astar;

use crate::{prelude::*, *};

/// A request to find a path between two points.
pub struct PathRequest<'a> {
    /// The starting point of the path.
    pub start: Vec2,
    /// The ending point of the path.
    pub end: Vec2,
    /// The radius of the agent requesting the path.
    pub radius: f32,
    /// How impatient the agent is (0.0 = patient, 1.0 = very impatient).
    /// This affects how much the agent avoids crowded areas.
    pub impatience: f32,
    /// The collision grid to use for pathfinding.
    pub grid: &'a NavigationGrid,
}

/// Finds a path from a start to an end point using A*.
/// The grid is used to determine walkable tiles.
pub fn find_path(request: PathRequest) -> Option<NavPath> {
    let (Some(start_tile), Some(end_tile)) = (
        request.grid.try_world_to_grid(request.start),
        request.grid.try_world_to_grid(request.end),
    ) else {
        return None; // Out of bounds
    };

    // Custom neighbor generator: allow entering the goal tile even if currently occupied.
    // This avoids unbounded search when the end cell is temporarily blocked by another diner.
    let neighbor_fn = |p: UVec2| {
        const DIRS: &[IVec2] = &[
            ivec2(-1, -1),
            ivec2(-1, 0),
            ivec2(-1, 1),
            ivec2(0, -1),
            ivec2(0, 1),
            ivec2(1, -1),
            ivec2(1, 0),
            ivec2(1, 1),
        ];

        let mut neighbors = Vec::with_capacity(8);
        for &d in DIRS {
            if let Some(cell) = request.grid.bound_tile(p.as_ivec2() + d)
                && request.grid.is_traversable(cell, request.radius)
            {
                let base = if d.x == 0 || d.y == 0 { 100 } else { 141 }; // Diagonal cost ~ sqrt(2)*100
                let extra = request.grid.crowd.sample(cell) * request.impatience * 100.0;
                let cost = base + extra.floor() as i32;
                neighbors.push((cell, cost));
            }
        }

        neighbors
    };

    let result = astar(
        &start_tile,
        |&p| neighbor_fn(p),
        |&p| p.manhattan_distance(end_tile) as i32,
        |&p| p == end_tile,
    );

    result.map(|(path, _cost)| {
        NavPath::new(
            path.into_iter()
                .map(|tile_pos| request.grid.tile_to_world(tile_pos))
                .collect(),
        )
    })
}
