//! Collision detection and spatial algorithms
//!
//! Contains spatial hash grids and collision detection utilities
//! for efficient proximity queries and physics simulation.

use rustc_hash::FxHashMap;

use crate::prelude::*;

/// Simple entity identifier wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionEntity(pub u64);

/// Axis-aligned bounding box collider for spatial collision detection
#[derive(Debug, Clone, Copy)]
pub struct BoxCollider {
    /// Center position of the collider
    pub center: Vec2,
    /// Width and height dimensions of the collider
    pub size: Vec2,
}

impl BoxCollider {
    /// Converts the collider to a Bevy Rect for intersection testing
    pub fn extent(&self) -> Rect {
        Rect::from_center_size(self.center, self.size)
    }
}

/// Spatial hash grid for efficient collision detection and proximity queries
///
/// Divides the game world into uniform grid cells to accelerate collision
/// detection and spatial queries. This avoids O(n²) entity-vs-entity checks
/// by only testing entities within the same or neighboring grid cells.
#[derive(Debug)]
pub struct CollisionGrid {
    /// Spatial hash map storing entity lists per grid cell coordinate
    cells: FxHashMap<IVec2, Vec<CollisionEntity>>,
    /// Size of each grid cell in world units
    cell_size: f32,
    /// Cache of all active colliders for direct entity-to-collider lookup
    colliders: FxHashMap<CollisionEntity, BoxCollider>,
}

impl CollisionGrid {
    /// Create a new collision grid with default cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cells: FxHashMap::default(),
            cell_size,
            colliders: FxHashMap::default(),
        }
    }

    /// Get the grid cell size in world units
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Convert world coordinates to grid cell coordinates
    pub fn world_to_grid(&self, position: Vec2) -> IVec2 {
        (position / self.cell_size).floor().as_ivec2()
    }

    /// Convert grid cell coordinates back to world position (cell center)
    pub fn tile_to_world(&self, tile_pos: IVec2) -> Vec2 {
        tile_pos.as_vec2() * self.cell_size + self.cell_size / 2.0
    }

    /// Calculate all grid cells that a rectangular object occupies
    pub fn get_occupied_cells(&self, center: Vec2, size: Vec2) -> Vec<IVec2> {
        let half_size = size / 2.0;
        let min = center - half_size;
        let max = center + half_size;

        let min_cell = self.world_to_grid(min);
        let max_cell = self.world_to_grid(max);

        let mut cells = Vec::new();
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                cells.push(IVec2::new(x, y));
            }
        }
        cells
    }

    /// Check if a specific grid cell is occupied by any entity
    pub fn is_occupied(&self, coord: IVec2) -> bool {
        self.cells.contains_key(&coord)
    }

    /// Test if an entity can be placed at a position without colliding
    pub fn is_position_valid(&self, entity: CollisionEntity, center: Vec2, size: Vec2) -> bool {
        let test_collider = BoxCollider { center, size };

        // Check all cells this object would occupy
        for cell_coord in self.get_occupied_cells(center, size) {
            let Some(entities) = self.cells.get(&cell_coord) else {
                continue;
            };
            for &other_entity in entities {
                if other_entity == entity {
                    continue; // Skip self
                }
                if let Some(other_collider) = self.colliders.get(&other_entity) {
                    let overlap = test_collider.extent().intersect(other_collider.extent());
                    if !overlap.is_empty() {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Find all entities within a circular radius of a center point
    pub fn find_nearby_entities(&self, center: Vec2, radius: f32) -> Vec<CollisionEntity> {
        let mut nearby = Vec::new();
        let radius_squared = radius * radius;

        // Check cells in a square around the center
        let center_cell = self.world_to_grid(center);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell_coord = center_cell + IVec2::new(dx, dy);
                let Some(entities) = self.cells.get(&cell_coord) else {
                    continue;
                };
                for &entity in entities {
                    if let Some(collider) = self.colliders.get(&entity) {
                        let distance_squared = center.distance_squared(collider.center);
                        if distance_squared <= radius_squared {
                            nearby.push(entity);
                        }
                    }
                }
            }
        }
        nearby
    }

    /// Get walkable neighboring grid cells for pathfinding
    pub fn get_neighbors(&self, pos: IVec2) -> Vec<(IVec2, i32)> {
        let mut neighbors = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let neighbor_pos = pos + IVec2::new(dx, dy);
                if !self.is_occupied(neighbor_pos) {
                    neighbors.push((neighbor_pos, 1));
                }
            }
        }
        neighbors
    }

    /// Get the collider for a specific entity, if present
    pub fn collider(&self, entity: CollisionEntity) -> Option<&BoxCollider> {
        self.colliders.get(&entity)
    }

    /// Collect occupied grid cells and the number of entities in each cell for debugging.
    pub fn debug_cells(&self) -> Vec<(IVec2, usize)> {
        self.cells
            .iter()
            .map(|(coord, entities)| (*coord, entities.len()))
            .collect()
    }

    /// Rebuild the spatial grid from current collider positions
    ///
    /// This should be called every frame or when entities move to keep
    /// the spatial hash accurate for collision detection.
    pub fn update<'a>(&mut self, query: impl Iterator<Item = (CollisionEntity, &'a BoxCollider)>) {
        // Clear the grid
        self.cells.clear();
        self.colliders.clear();

        // Rebuild from current query
        for (entity, collider) in query {
            // Store collider for quick lookup
            self.colliders.insert(entity, *collider);

            // Add to spatial grid
            for cell_coord in self.get_occupied_cells(collider.center, collider.size) {
                self.cells.entry(cell_coord).or_default().push(entity);
            }
        }
    }
}
