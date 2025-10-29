use std::{
    collections::{HashMap, HashSet},
    f32::consts::TAU,
};

use dishaster_interface::snapshots::MovementDebugSnapshot;
use dishrupt_core::{EntityId, prelude::*};
use dishrupt_godot::display::DisplayContext2D;
use godot::{
    classes::{Line2D, Node2D},
    prelude::*,
};

const POSITION_RADIUS: f32 = 0.05;
const VELOCITY_SCALE: f32 = 0.5;
const PATH_WIDTH: f32 = 2.0;
const VELOCITY_WIDTH: f32 = 2.5;
const MARKER_WIDTH: f32 = 3.0;
const Z_PATH: i32 = 1000;
const Z_VELOCITY: i32 = 1001;
const Z_MARKER: i32 = 1002;
const MARKER_SEGMENTS: usize = 4;

pub struct MovementDebugOverlay {
    root: Gd<Node2D>,
    nodes: HashMap<EntityId, MovementDebugNodes>,
}

struct MovementDebugNodes {
    path: Gd<Line2D>,
    velocity_line: Gd<Line2D>,
    marker: Gd<Line2D>,
}

impl MovementDebugOverlay {
    pub fn new(root: Gd<Node2D>) -> Self {
        Self {
            root,
            nodes: HashMap::new(),
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.root.set_visible(visible);
    }

    pub fn present(
        &mut self,
        snapshots: Option<&Vec<MovementDebugSnapshot>>,
        ctx: &DisplayContext2D,
    ) {
        let Some(snapshots) = snapshots else {
            return;
        };

        let mut seen = HashSet::new();
        for snapshot in snapshots {
            seen.insert(snapshot.core_id);
            if !self.nodes.contains_key(&snapshot.core_id) {
                let nodes = self.spawn_nodes(snapshot.core_id);
                self.nodes.insert(snapshot.core_id, nodes);
            }
            if let Some(nodes) = self.nodes.get_mut(&snapshot.core_id) {
                Self::update_nodes(nodes, snapshot, ctx);
            }
        }

        self.cleanup_unseen(&seen);
    }

    fn spawn_nodes(&mut self, entity: EntityId) -> MovementDebugNodes {
        let mut path = Line2D::new_alloc();
        path.set_name(&format!("MovementPath_{}", entity.to_bits()));
        path.set_default_color(Color::from_rgba(0.2, 0.8, 1.0, 0.9));
        path.set_width(PATH_WIDTH);
        path.set_z_index(Z_PATH);
        self.root.add_child(&path);

        let mut velocity = Line2D::new_alloc();
        velocity.set_name(&format!("MovementVelocity_{}", entity.to_bits()));
        velocity.set_default_color(Color::from_rgba(1.0, 0.3, 0.3, 0.9));
        velocity.set_width(VELOCITY_WIDTH);
        velocity.set_z_index(Z_VELOCITY);
        self.root.add_child(&velocity);

        let mut marker = Line2D::new_alloc();
        marker.set_name(&format!("MovementMarker_{}", entity.to_bits()));
        marker.set_default_color(Color::from_rgba(1.0, 0.95, 0.4, 0.95));
        marker.set_width(MARKER_WIDTH);
        marker.set_closed(true);
        marker.set_z_index(Z_MARKER);
        self.root.add_child(&marker);

        MovementDebugNodes {
            path,
            velocity_line: velocity,
            marker,
        }
    }

    fn update_nodes(
        nodes: &mut MovementDebugNodes,
        snapshot: &MovementDebugSnapshot,
        ctx: &DisplayContext2D,
    ) {
        let display_pos = ctx.to_display_space(snapshot.position.extend(0.0));
        Self::update_path(nodes, snapshot, ctx, display_pos);
        Self::update_velocity(nodes, snapshot, ctx, display_pos);
        Self::update_marker(nodes, snapshot, ctx);
    }

    fn update_path(
        nodes: &mut MovementDebugNodes,
        snapshot: &MovementDebugSnapshot,
        ctx: &DisplayContext2D,
        display_pos: Vector2,
    ) {
        let mut points = PackedVector2Array::new();
        for waypoint in &snapshot.path {
            points.push(ctx.to_display_space(waypoint.extend(0.0)));
        }
        points.push(display_pos);

        nodes.path.set_points(&points);
        nodes.path.set_visible(!snapshot.path.is_empty());
    }

    fn update_velocity(
        nodes: &mut MovementDebugNodes,
        snapshot: &MovementDebugSnapshot,
        ctx: &DisplayContext2D,
        display_pos: Vector2,
    ) {
        if snapshot.velocity.length_squared() <= f32::EPSILON {
            nodes.velocity_line.set_visible(false);
            return;
        }

        let target = snapshot.position + snapshot.velocity.normalize() * VELOCITY_SCALE;
        let display_target = ctx.to_display_space(target.extend(0.0));

        let mut points = PackedVector2Array::new();
        points.push(display_pos);
        points.push(display_target);

        nodes.velocity_line.set_points(&points);
        nodes.velocity_line.set_visible(true);
    }

    fn update_marker(
        nodes: &mut MovementDebugNodes,
        snapshot: &MovementDebugSnapshot,
        ctx: &DisplayContext2D,
    ) {
        let mut points = PackedVector2Array::new();

        for step in 0..MARKER_SEGMENTS {
            let ratio = step as f32 / MARKER_SEGMENTS as f32;
            let angle = ratio * TAU;
            let offset = Vec2::from_angle(angle) * POSITION_RADIUS;
            let sample = snapshot.position + offset;
            let display = ctx.to_display_space(sample.extend(0.0));
            points.push(display);
        }
        if let Some(first) = points.get(0) {
            points.push(first);
        }

        nodes.marker.set_points(&points);
        nodes.marker.set_visible(true);
    }

    fn cleanup_unseen(&mut self, seen: &HashSet<EntityId>) {
        self.nodes.retain(|entity, nodes| {
            if seen.contains(entity) {
                return true;
            }
            nodes.path.queue_free();
            nodes.velocity_line.queue_free();
            nodes.marker.queue_free();
            false
        });
    }
}
