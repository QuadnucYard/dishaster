use std::collections::{HashMap, HashSet};

use dishaster_core::snapshots::{CollisionCellDebugSnapshot, CollisionGridDebugSnapshot};
use dishrupt_core::prelude::*;
use dishrupt_godot::display::DisplayContext2D;
use godot::{
    classes::{Line2D, Node2D},
    prelude::*,
};

const BASE_WIDTH: f32 = 2.0;
const Z_INDEX: i32 = 6;

pub struct CollisionDebugOverlay {
    root: Gd<Node2D>,
    cells: HashMap<IVec2, Gd<Line2D>>,
}

impl CollisionDebugOverlay {
    pub fn new(mut root: Gd<Node2D>) -> Self {
        root.set_z_index(Z_INDEX);
        Self {
            root,
            cells: HashMap::new(),
        }
    }

    pub fn present(
        &mut self,
        snapshot: Option<&CollisionGridDebugSnapshot>,
        ctx: &DisplayContext2D,
    ) {
        let Some(snapshot) = snapshot else {
            self.clear();
            return;
        };

        let mut seen = HashSet::new();
        for cell in &snapshot.cells {
            seen.insert(cell.coord);
            if !self.cells.contains_key(&cell.coord) {
                let node = self.spawn_cell(cell.coord);
                self.cells.insert(cell.coord, node);
            }
            if let Some(node) = self.cells.get_mut(&cell.coord) {
                Self::update_cell(node, cell, ctx);
            }
        }

        self.cleanup_unseen(&seen);
    }

    fn spawn_cell(&mut self, coord: IVec2) -> Gd<Line2D> {
        let mut node = Line2D::new_alloc();
        node.set_name(&format!("CollisionCell_{}_{}", coord.x, coord.y));
        node.set_antialiased(true);
        node.set_width(BASE_WIDTH);
        node.set_z_index(Z_INDEX);
        self.root.add_child(&node);
        node
    }

    fn update_cell(
        node: &mut Gd<Line2D>,
        cell: &CollisionCellDebugSnapshot,
        ctx: &DisplayContext2D,
    ) {
        let mut points = PackedVector2Array::new();
        let half = cell.size * 0.5;
        let corners = [
            Vec3::new(cell.center.x - half.x, cell.center.y - half.y, 0.0),
            Vec3::new(cell.center.x + half.x, cell.center.y - half.y, 0.0),
            Vec3::new(cell.center.x + half.x, cell.center.y + half.y, 0.0),
            Vec3::new(cell.center.x - half.x, cell.center.y + half.y, 0.0),
        ];

        for corner in &corners {
            points.push(ctx.to_display_space(*corner));
        }
        if let Some(first) = points.get(0) {
            points.push(first);
        }

        let occupancy = cell.occupancy as f32;
        let intensity = (occupancy / 5.0).clamp(0.0, 1.0);
        let color = Color::from_rgba(1.0, 0.6 - 0.4 * intensity, 0.2 + 0.3 * intensity, 0.75);

        node.set_points(&points);
        node.set_default_color(color);
        node.set_width(BASE_WIDTH * (0.75 + intensity * 1.25));
        node.set_visible(true);
    }

    fn cleanup_unseen(&mut self, seen: &HashSet<IVec2>) {
        self.cells.retain(|coord, node| {
            if seen.contains(coord) {
                return true;
            }
            node.queue_free();
            false
        });
    }

    fn clear(&mut self) {
        for node in self.cells.values_mut() {
            node.queue_free();
        }
        self.cells.clear();
    }
}
