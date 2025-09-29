use dodgy::Obstacle;
use grid::Grid;

use crate::{BoxCollider, CrowdCostField, avoidance::IntoDodgy, prelude::*};

/// Static navigation grid for spatial partitioning and proximity queries
#[derive(Debug)]
pub struct NavigationGrid {
    /// Size of each grid cell in world units
    cell_size: f32,
    /// Number of cells in each dimension
    grid_size: USizeVec2,
    /// Occupancy grid for quick lookup of occupied cells
    occupancy: Grid<bool>,
    /// Distance field for obstacle avoidance
    distance: Grid<f64>,
    /// Obstacles for use with the dodgy library
    obstacles: Vec<Obstacle>,
    /// Crowd cost field for soft avoidance of other agents
    pub crowd: CrowdCostField,
}

impl NavigationGrid {
    const CLEARANCE_EPS: f32 = -0.1;

    /// Create a new collision grid with default cell size
    pub fn new(world_size: Vec2, cell_size: f32) -> Self {
        let grid_size = USizeVec2::new(
            (world_size.x / cell_size).ceil() as usize,
            (world_size.y / cell_size).ceil() as usize,
        );
        Self {
            cell_size,
            grid_size,
            occupancy: Grid::new(grid_size.x, grid_size.y),
            distance: Grid::new(grid_size.x, grid_size.y),
            obstacles: Default::default(),
            crowd: CrowdCostField::new(world_size, cell_size),
        }
    }

    /// Get all obstacles in the grid
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    /// Get the grid cell size in world units
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Ensure a tile coordinate is within grid bounds
    pub fn bound_tile(&self, tile: IVec2) -> Option<UVec2> {
        if tile.x < 0 || tile.y < 0 {
            return None;
        }
        if tile.x >= self.grid_size.x as i32 || tile.y >= self.grid_size.y as i32 {
            return None;
        }
        Some(tile.as_uvec2())
    }

    /// Convert world coordinates to grid cell coordinates
    pub fn world_to_igrid(&self, position: Vec2) -> IVec2 {
        (position / self.cell_size).floor().as_ivec2()
    }

    /// Convert world coordinates to grid cell coordinates
    pub fn world_to_grid(&self, position: Vec2) -> UVec2 {
        (position / self.cell_size).floor().as_uvec2()
    }

    /// Convert world coordinates to grid cell coordinates
    pub fn try_world_to_grid(&self, position: Vec2) -> Option<UVec2> {
        self.bound_tile((position / self.cell_size).floor().as_ivec2())
    }

    /// Convert grid cell coordinates back to world position (cell center)
    pub fn tile_to_world(&self, tile_pos: UVec2) -> Vec2 {
        tile_pos.as_vec2() * self.cell_size + self.cell_size / 2.0
    }

    /// Calculate all grid cells that a rectangular object occupies
    pub fn get_occupied_cells(&self, center: Vec2, size: Vec2) -> Vec<UVec2> {
        let half_size = size / 2.0;
        let min = center - half_size;
        let max = center + half_size;

        let min_cell = self.world_to_grid(min);
        let max_cell = self.world_to_grid(max);

        let mut cells = Vec::new();
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                cells.push(UVec2::new(x, y));
            }
        }
        cells
    }

    /// Check if a specific grid cell is occupied by any entity
    pub fn is_occupied(&self, coord: UVec2) -> bool {
        self.occupancy[(coord.x as usize, coord.y as usize)]
    }

    /// Check if a cell is traversable given an entity radius
    pub fn is_traversable(&self, cell: UVec2, radius: f32) -> bool {
        (self.distance[(cell.x as usize, cell.y as usize)] as f32) * self.cell_size
            > radius + Self::CLEARANCE_EPS
    }

    /// Rebuild the spatial grid from current collider positions
    ///
    /// This should be called every frame or when entities move to keep
    /// the spatial hash accurate for collision detection.
    pub fn update<'a>(&mut self, colliders: impl Iterator<Item = &'a BoxCollider>) {
        // Clear the grid
        self.occupancy.fill(false);
        self.obstacles.clear();

        // Rebuild from current query
        for collider in colliders {
            // Store collider for quick lookup
            self.obstacles.push(Obstacle::Closed {
                vertices: vec![
                    (collider.center + Vec2::new(-collider.size.x / 2.0, -collider.size.y / 2.0))
                        .into_dodgy(),
                    (collider.center + Vec2::new(collider.size.x / 2.0, -collider.size.y / 2.0))
                        .into_dodgy(),
                    (collider.center + Vec2::new(collider.size.x / 2.0, collider.size.y / 2.0))
                        .into_dodgy(),
                    (collider.center + Vec2::new(-collider.size.x / 2.0, collider.size.y / 2.0))
                        .into_dodgy(),
                ],
            });

            // Add to spatial grid
            for cell_coord in self.get_occupied_cells(collider.center, collider.size) {
                let cell_coord = cell_coord.as_usizevec2();
                if cell_coord.x >= self.grid_size.x || cell_coord.y >= self.grid_size.y {
                    continue; // Out of bounds
                }
                self.occupancy[(cell_coord.x, cell_coord.y)] = true;
            }
        }

        // Recompute distance field for obstacle avoidance
        self.distance = Grid::from_vec(
            edt::edt(
                self.occupancy.iter().as_slice(),
                (self.grid_size.x, self.grid_size.y),
                true,
            ),
            self.grid_size.y,
        );
    }
}
