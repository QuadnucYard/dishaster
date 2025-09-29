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
        let mut neighbors = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if let Some(cell) = request
                    .grid
                    .bound_tile(IVec2::new(p.x as i32 + dx, p.y as i32 + dy))
                    && request.grid.is_traversable(cell, request.radius)
                {
                    let cost = if dx == 0 || dy == 0 { 100 } else { 141 }; // Diagonal cost ~ sqrt(2)*100
                    neighbors.push((cell, cost));
                }
            }
        }
        neighbors
    };

    let result = astar(
        &start_tile,
        |&p| {
            let neighbors = neighbor_fn(p);
            neighbors
                .into_iter()
                .map(|(n, base)| {
                    let extra = request.grid.crowd.sample(n) * 100.0;
                    let cost = base + (extra.ceil() as i32).max(0);

                    (n, cost)
                })
                .collect::<Vec<_>>()
        },
        |&p| (p.x.abs_diff(end_tile.x) + p.y.abs_diff(end_tile.y)) as i32,
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
