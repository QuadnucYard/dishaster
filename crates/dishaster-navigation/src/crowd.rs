//! Crowd cost field for pathfinding soft-avoidance
//!
//! We compute a per-tile additional traversal cost based on nearby diners.
//! This encourages paths that keep a certain distance from other agents without
//! hard constraints.

use grid::Grid;

use crate::prelude::*;

/// Tile coordinate in the collision grid
pub type Tile = IVec2;

type Cost = f32;

/// Crowd cost field stored on a fixed-size tile grid covering the world.
#[derive(Debug)]
pub struct CrowdCostField {
    grid: Grid<Cost>,
    cell_size: f32,
    tile_dims: USizeVec2,
    origin: IVec2,
}

impl CrowdCostField {
    /// Create a new crowd field for the provided world size.
    pub fn new(world_width: f32, world_height: f32, cell_size: f32) -> Self {
        let cell_size = cell_size.max(0.001);
        let dims = Self::tile_counts(world_width, world_height, cell_size);

        Self {
            grid: Grid::new(dims.y, dims.x),
            cell_size,
            tile_dims: dims,
            origin: IVec2::ZERO,
        }
    }

    /// Clear all accumulated costs.
    pub fn clear(&mut self) {
        self.grid.fill(Cost::default());
    }

    /// Add cost to a tile (accumulated)
    pub fn add_cost(&mut self, tile: Tile, extra: Cost) {
        if extra <= 0.0 {
            return;
        }
        if let Some((row, col)) = self.index(tile) {
            self.grid[(row, col)] += extra;
        }
    }

    /// Sample cost at a tile (0 if none)
    pub fn sample(&self, tile: Tile) -> Cost {
        self.index(tile)
            .map(|(row, col)| self.grid[(row, col)])
            .unwrap_or_default()
    }

    /// Get the grid cell size in world units
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Collect crowd cost entries for debugging visualization.
    pub fn costs(&self) -> &Grid<Cost> {
        &self.grid
    }

    /// Bounds of the covered tile grid.
    pub fn tile_bounds(&self) -> Option<(IVec2, IVec2)> {
        if self.tile_dims.x == 0 || self.tile_dims.y == 0 {
            return None;
        }
        let max = self.origin + self.tile_dims.as_ivec2();
        Some((self.origin, max))
    }

    /// Tile dimensions (width, height) of the grid.
    pub fn tile_dimensions(&self) -> USizeVec2 {
        self.tile_dims
    }

    fn tile_counts(world_width: f32, world_height: f32, cell_size: f32) -> USizeVec2 {
        USizeVec2::new(
            ((world_width / cell_size).ceil() as usize).max(1),
            ((world_height / cell_size).ceil() as usize).max(1),
        )
    }

    fn index(&self, tile: Tile) -> Option<(usize, usize)> {
        if tile.x < self.origin.x || tile.y < self.origin.y {
            return None;
        }
        let rel_x = tile.x - self.origin.x;
        let rel_y = tile.y - self.origin.y;
        if rel_x < 0
            || rel_x >= self.tile_dims.x as i32
            || rel_y < 0
            || rel_y >= self.tile_dims.y as i32
        {
            return None;
        }
        Some((rel_y as usize, rel_x as usize))
    }
}

/// Convert world distance to tile distance helper
#[inline]
pub fn world_to_tile_dist(d: f32, cell: f32) -> f32 {
    d / cell
}
