//! Pathfinding utilities using the `pathfinding` crate.

use ::pathfinding::prelude::astar;

use crate::{prelude::*, *};

/// Target for pathfinding requests
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathTarget {
    /// A specific point in the world
    Point(Vec2),
    /// A rectangular area in the world
    Rect(Rect),
}

impl std::fmt::Display for PathTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathTarget::Point(p) => p.fmt(f),
            PathTarget::Rect(r) => write!(f, "{:?}", r),
        }
    }
}

/// A request to find a path between two points.
pub struct PathRequest<'a> {
    /// The starting point of the path.
    pub start: Vec2,
    /// The ending point of the path.
    pub target: PathTarget,
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
    match request.target {
        PathTarget::Point(target) => find_path_to_point(request, target),
        PathTarget::Rect(target) => find_path_to_rect(request, target),
    }
}

fn find_path_to_point(request: PathRequest, target: Vec2) -> Option<NavPath> {
    let start_tile = request.grid.try_world_to_grid(request.start)?;
    let end_tile = request.grid.try_world_to_grid(target)?;

    let neighbor_fn = create_neighbor_fn(request.grid, request.radius, request.impatience);

    let result = astar(
        &start_tile,
        |&p| neighbor_fn(p),
        |&p| p.manhattan_distance(end_tile) as i32,
        |&p| p == end_tile,
    );

    result.map(|(path, _)| create_path(path, request.grid))
}

fn find_path_to_rect(request: PathRequest, target: Rect) -> Option<NavPath> {
    let start_tile = request.grid.try_world_to_grid(request.start)?;
    let end_center = request.grid.try_world_to_grid(target.center())?;
    let end_rect = {
        let min = request.grid.try_world_to_grid(target.min)?;
        let max = request.grid.try_world_to_grid(target.max)?;
        URect::from_corners(min, max)
    };

    let neighbor_fn = create_neighbor_fn(request.grid, request.radius, request.impatience);

    let result = astar(
        &start_tile,
        |&p| neighbor_fn(p),
        |&p| p.manhattan_distance(end_center) as i32,
        |&p| end_rect.contains(p),
    );

    result.map(|(path, _)| create_path(path, request.grid))
}

fn create_neighbor_fn(
    grid: &NavigationGrid,
    radius: f32,
    impatience: f32,
) -> impl Fn(UVec2) -> Vec<(UVec2, i32)> {
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

    move |p: UVec2| {
        let mut neighbors = Vec::with_capacity(8);
        for &d in DIRS {
            if let Some(cell) = grid.bound_tile(p.as_ivec2() + d)
                && grid.is_traversable(cell, radius)
            {
                let base = if d.x == 0 || d.y == 0 { 100 } else { 141 }; // Diagonal cost ~ sqrt(2)*100
                let extra = grid.crowd.sample(cell) * impatience * 100.0;
                let cost = base + extra.floor() as i32;
                neighbors.push((cell, cost));
            }
        }

        neighbors
    }
}

fn create_path(path: Vec<UVec2>, grid: &NavigationGrid) -> NavPath {
    NavPath::new(
        path.into_iter()
            .map(|tile_pos| grid.tile_to_world(tile_pos))
            .collect(),
    )
}
