//! Crowd cost field for pathfinding soft-avoidance
//!
//! We compute a per-tile additional traversal cost based on nearby diners.
//! This encourages paths that keep a certain distance from other agents without
//! hard constraints.

use grid::Grid;

use crate::prelude::*;

/// Tile coordinate in the collision grid
pub type Tile = UVec2;

type Cost = f32;

/// Crowd cost field stored on a fixed-size tile grid covering the world.
#[derive(Debug)]
pub struct CrowdCostField {
    grid: Grid<Cost>,
    cell_size: f32,
    tile_dims: USizeVec2,
}

impl CrowdCostField {
    /// Create a new crowd field for the provided world size.
    pub fn new(world_size: Vec2, cell_size: f32) -> Self {
        assert!(
            world_size.x > 0.0 && world_size.y > 0.0,
            "world size must be positive"
        );
        assert!(cell_size > 0.0, "cell size must be positive");
        let dims = Self::tile_counts(world_size.x, world_size.y, cell_size);

        Self {
            grid: Grid::new(dims.y, dims.x),
            cell_size,
            tile_dims: dims,
        }
    }

    /// Clear all accumulated costs.
    pub fn clear(&mut self) {
        self.grid.fill(Cost::default());
    }

    /// Add cost to a tile (accumulated).
    ///
    /// Each agent contributes cost to nearby tiles based on distance.
    /// Multiple agents' costs accumulate, creating a continuous density field.
    pub fn add_cost(&mut self, tile: Tile, extra: Cost) {
        if extra <= 0.0 {
            return;
        }
        if let Some(idx) = self.index(tile) {
            self.grid[idx] += extra;
        }
    }

    /// Sample raw cost at a tile (0 if none).
    ///
    /// Returns unnormalized cost value. Use `sample_normalized()` for
    /// density values suitable for speed calculations.
    pub fn sample(&self, tile: Tile) -> Cost {
        self.index(tile)
            .map(|idx| self.grid[idx])
            .unwrap_or_default()
    }

    /// Get normalized crowd density at a tile in range [0, 1].
    ///
    /// **Algorithm: Fixed Reference Normalization**
    /// - Uses a fixed reference value representing "moderate congestion"
    /// - cost=0 → density=0.0 (empty)
    /// - cost=expected_moderate → density≈0.5 (moderate)
    /// - cost>>expected → density→1.0 (saturates at very high density)
    ///
    /// Formula: density = cost / (cost + reference)
    /// This creates a smooth curve that:
    /// - Responds sensitively to low/medium densities
    /// - Saturates gracefully at high densities (no unbounded values)
    /// - Doesn't depend on the current maximum (stable across different crowd sizes)
    ///
    /// **Example with reference=5.0:**
    /// - cost=0 → 0/(0+5) = 0.0 (empty)
    /// - cost=2.5 → 2.5/7.5 = 0.33 (light crowd)
    /// - cost=5.0 → 5.0/10 = 0.5 (moderate)
    /// - cost=15.0 → 15/20 = 0.75 (heavy)
    /// - cost=45.0 → 45/50 = 0.9 (very dense)
    pub fn sample_normalized(&self, tile: Tile) -> f32 {
        // Reference cost representing "moderate congestion" level
        // This is based on typical agent influence: radius=0.3m, max_extra=3.0
        // At moderate distance, an agent contributes ~5.0 cost to nearby tiles
        const REFERENCE_COST: f32 = 5.0;

        let cost = self.sample(tile);
        cost / (cost + REFERENCE_COST)
    }

    /// Get the maximum cost value in the grid
    pub fn max_value(&self) -> f32 {
        self.grid.iter().cloned().fold(0.0, f32::max)
    }

    /// Get the grid cell size in world units
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Collect crowd cost entries for debugging visualization.
    pub fn costs(&self) -> &Grid<Cost> {
        &self.grid
    }

    /// Tile dimensions (width, height) of the grid.
    pub fn tile_dimensions(&self) -> USizeVec2 {
        self.tile_dims
    }

    fn tile_counts(world_width: f32, world_height: f32, cell_size: f32) -> USizeVec2 {
        USizeVec2::new(
            (world_width / cell_size).ceil() as usize,
            (world_height / cell_size).ceil() as usize,
        )
    }

    fn index(&self, tile: Tile) -> Option<(usize, usize)> {
        if tile.x >= self.tile_dims.x as u32 || tile.y >= self.tile_dims.y as u32 {
            return None;
        }
        Some((tile.y as usize, tile.x as usize))
    }
}

/// Convert world distance to tile distance helper
#[inline]
pub fn world_to_tile_dist(d: f32, cell: f32) -> f32 {
    d / cell
}
