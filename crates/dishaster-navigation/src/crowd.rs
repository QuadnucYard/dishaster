//! Crowd cost field for pathfinding soft-avoidance
//!
//! We compute a per-tile additional traversal cost based on nearby diners.
//! This encourages paths that keep a certain distance from other agents without
//! hard constraints.

use rustc_hash::FxHashMap;

use crate::prelude::*;

/// Tile coordinate in the collision grid
pub type Tile = IVec2;

/// Crowd cost field stored sparsely as additional traversal cost per tile
#[derive(Default, Debug)]
pub struct CrowdCostField {
    costs: FxHashMap<Tile, f32>,
    cell_size: f32,
}

impl CrowdCostField {
    /// Constructor with specified grid cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            costs: FxHashMap::default(),
            cell_size,
        }
    }

    /// Clear all costs
    pub fn clear(&mut self) {
        self.costs.clear();
    }

    /// Add cost to a tile (accumulated)
    pub fn add_cost(&mut self, tile: Tile, extra: f32) {
        *self.costs.entry(tile).or_insert(0.0) += extra.max(0.0);
    }

    /// Sample cost at a tile (0 if none)
    pub fn sample(&self, tile: Tile) -> f32 {
        self.costs.get(&tile).copied().unwrap_or(0.0)
    }

    /// Get the grid cell size in world units
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Update the grid cell size (used when CollisionGrid changes configuration)
    pub fn set_cell_size(&mut self, cell_size: f32) {
        self.cell_size = cell_size;
    }
}

/// Convert world distance to tile distance helper
#[inline]
pub fn world_to_tile_dist(d: f32, cell: f32) -> f32 {
    d / cell
}
