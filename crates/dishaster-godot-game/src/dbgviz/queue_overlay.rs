use std::collections::{HashMap, HashSet};

use dishaster_interface::snapshots::{
    QueueIntentDebugSnapshot, QueueLaneDebugSnapshot, QueueMemberDebugSnapshot,
};
use dishrupt_core::EntityId;
use dishrupt_godot_display::DisplayContext2D;
use godot::{
    classes::{Line2D, Node2D},
    prelude::*,
};

const MEMBER_RADIUS: f32 = 5.0;
const MEMBER_SEGMENTS: usize = 8;
const INTENT_RADIUS: f32 = 5.0;
const PATH_WIDTH: f32 = 4.0;
const CONNECTOR_WIDTH: f32 = 2.8;
const MARKER_WIDTH: f32 = 5.0;
const INTENT_MARKER_WIDTH: f32 = 4.0;
const REAR_MARKER_SIZE: f32 = 8.0;
const REAR_MARKER_WIDTH: f32 = 3.0;
const Z_PATH: i32 = 1100;
const Z_CONNECTOR: i32 = 1101;
const Z_MEMBER_MARKER: i32 = 1102;
const Z_INTENT_MARKER: i32 = 1103;
const Z_REAR_MARKER: i32 = 1104;
use std::f32::consts::TAU;

const COLOR_PATH: Color = Color::from_rgba(0.15, 0.9, 0.45, 0.9);
const COLOR_MEMBER_MARKER: Color = Color::from_rgba(1.0, 0.92, 0.4, 0.95);
const COLOR_CONNECTOR: Color = Color::from_rgba(0.8, 0.35, 1.0, 0.8);
const COLOR_INTENT_MARKER: Color = Color::from_rgba(1.0, 0.6, 0.2, 0.85);
const COLOR_REAR_MARKER: Color = Color::from_rgba(1.0, 0.1, 0.1, 0.95);

/// Visualizes queue lane membership and intents on the debug canvas.
pub struct QueueDebugOverlay {
    root: Gd<Node2D>,
    lanes: HashMap<EntityId, QueueLaneDebugNodes>,
}

struct QueueLaneDebugNodes {
    container: Gd<Node2D>,
    path: Gd<Line2D>,
    rear_marker: Gd<Line2D>,
    member_markers: Vec<Gd<Line2D>>,
    intent_connectors: Vec<Gd<Line2D>>,
    intent_markers: Vec<Gd<Line2D>>,
}

impl QueueDebugOverlay {
    /// Create the overlay attached to the provided Godot node.
    pub fn new(mut root: Gd<Node2D>) -> Self {
        root.set_z_index(101);
        Self {
            root,
            lanes: HashMap::new(),
        }
    }

    /// Render the current queue debug snapshot onto the Godot canvas.
    pub fn present(
        &mut self,
        snapshot: Option<&Vec<QueueLaneDebugSnapshot>>,
        ctx: &DisplayContext2D,
    ) {
        let Some(lanes) = snapshot else {
            self.hide_all();
            return;
        };

        let mut seen = HashSet::new();
        for lane_snapshot in lanes {
            seen.insert(lane_snapshot.lane_id);
            let lane_id = lane_snapshot.lane_id;
            if !self.lanes.contains_key(&lane_id) {
                let nodes = self.spawn_lane_nodes(lane_id);
                self.lanes.insert(lane_id, nodes);
            }
            if let Some(nodes) = self.lanes.get_mut(&lane_id) {
                Self::update_lane(nodes, lane_snapshot, ctx);
            }
        }

        self.cleanup_unseen(&seen);
    }

    fn spawn_lane_nodes(&mut self, lane_id: EntityId) -> QueueLaneDebugNodes {
        let mut container = Node2D::new_alloc();
        container.set_name(&format!("QueueLane_{}", lane_id));
        self.root.add_child(&container);

        let mut path = Line2D::new_alloc();
        path.set_name(&format!("QueuePath_{}", lane_id));
        path.set_default_color(COLOR_PATH);
        path.set_width(PATH_WIDTH);
        path.set_z_index(Z_PATH);
        container.add_child(&path);

        let mut rear_marker = Line2D::new_alloc();
        rear_marker.set_name(&format!("QueueRear_{}", lane_id));
        rear_marker.set_default_color(COLOR_REAR_MARKER);
        rear_marker.set_width(REAR_MARKER_WIDTH);
        rear_marker.set_z_index(Z_REAR_MARKER);
        container.add_child(&rear_marker);

        QueueLaneDebugNodes {
            container,
            path,
            rear_marker,
            member_markers: Vec::new(),
            intent_connectors: Vec::new(),
            intent_markers: Vec::new(),
        }
    }

    fn update_lane(
        nodes: &mut QueueLaneDebugNodes,
        snapshot: &QueueLaneDebugSnapshot,
        ctx: &DisplayContext2D,
    ) {
        nodes.container.set_visible(true);
        let rear = ctx.to_display_space(snapshot.rear_pos.extend(0.0));
        Self::update_rear_marker(&mut nodes.rear_marker, rear);
        Self::update_path(&mut nodes.path, snapshot, ctx);
        Self::update_members(nodes, snapshot.lane_id, &snapshot.members, ctx);
        Self::update_intents(nodes, snapshot.lane_id, rear, &snapshot.intents, ctx);
    }

    fn update_rear_marker(marker: &mut Gd<Line2D>, rear: Vector2) {
        // Draw a cross/X marker at the rear position
        let mut points = PackedVector2Array::new();
        let half_size = REAR_MARKER_SIZE;
        points.push(rear + Vector2::new(-half_size, -half_size));
        points.push(rear + Vector2::new(half_size, half_size));
        points.push(rear);
        points.push(rear + Vector2::new(-half_size, half_size));
        points.push(rear + Vector2::new(half_size, -half_size));
        marker.set_points(&points);
        marker.set_visible(true);
    }

    fn update_path(
        path: &mut Gd<Line2D>,
        snapshot: &QueueLaneDebugSnapshot,
        ctx: &DisplayContext2D,
    ) {
        let mut points = PackedVector2Array::new();
        points.push(ctx.to_display_space(snapshot.anchor.extend(0.0)));
        for member in &snapshot.members {
            let display = ctx.to_display_space(member.position.extend(0.0));
            points.push(display);
        }
        points.push(ctx.to_display_space(snapshot.rear_pos.extend(0.0)));

        path.set_points(&points);
        path.set_visible(points.len() > 1);
    }

    fn update_members(
        nodes: &mut QueueLaneDebugNodes,
        lane_id: EntityId,
        members: &[QueueMemberDebugSnapshot],
        ctx: &DisplayContext2D,
    ) {
        while nodes.member_markers.len() < members.len() {
            let marker = Self::create_member_marker(lane_id, nodes.member_markers.len());
            nodes.container.add_child(&marker);
            nodes.member_markers.push(marker);
        }
        for marker in nodes.member_markers.iter_mut().skip(members.len()) {
            marker.set_visible(false);
        }

        for (marker, member) in nodes.member_markers.iter_mut().zip(members.iter()) {
            let display = ctx.to_display_space(member.position.extend(0.0));
            let mut points = PackedVector2Array::new();
            for step in 0..MEMBER_SEGMENTS {
                let angle = (step as f32 / MEMBER_SEGMENTS as f32) * TAU;
                let offset = Vector2::from_angle(angle) * MEMBER_RADIUS;
                points.push(display + offset);
            }
            if let Some(first) = points.get(0) {
                points.push(first);
            }

            marker.set_points(&points);
            marker.set_visible(true);
        }
    }

    fn update_intents(
        nodes: &mut QueueLaneDebugNodes,
        lane_id: EntityId,
        rear: Vector2,
        intents: &[QueueIntentDebugSnapshot],
        ctx: &DisplayContext2D,
    ) {
        while nodes.intent_connectors.len() < intents.len() {
            let connector = Self::create_intent_connector(lane_id, nodes.intent_connectors.len());
            nodes.container.add_child(&connector);
            nodes.intent_connectors.push(connector);
        }
        for connector in nodes.intent_connectors.iter_mut().skip(intents.len()) {
            connector.set_visible(false);
        }

        while nodes.intent_markers.len() < intents.len() {
            let marker = Self::create_intent_marker(lane_id, nodes.intent_markers.len());
            nodes.container.add_child(&marker);
            nodes.intent_markers.push(marker);
        }
        for marker in nodes.intent_markers.iter_mut().skip(intents.len()) {
            marker.set_visible(false);
        }

        for (idx, intent) in intents.iter().enumerate() {
            let connector = nodes
                .intent_connectors
                .get_mut(idx)
                .expect("intent connector missing");
            let marker = nodes
                .intent_markers
                .get_mut(idx)
                .expect("intent marker missing");
            let display = ctx.to_display_space(intent.position.extend(0.0));
            let mut connector_points = PackedVector2Array::new();
            connector_points.push(rear);
            connector_points.push(display);
            connector.set_points(&connector_points);
            connector.set_visible(true);

            let mut marker_points = PackedVector2Array::new();
            marker_points.push(display + Vector2::new(0.0, INTENT_RADIUS));
            marker_points.push(display + Vector2::new(INTENT_RADIUS, 0.0));
            marker_points.push(display + Vector2::new(0.0, -INTENT_RADIUS));
            marker_points.push(display + Vector2::new(-INTENT_RADIUS, 0.0));
            marker_points.push(display + Vector2::new(0.0, INTENT_RADIUS));
            marker.set_points(&marker_points);
            marker.set_visible(true);
        }
    }

    fn create_member_marker(lane_id: EntityId, index: usize) -> Gd<Line2D> {
        let mut marker = Line2D::new_alloc();
        marker.set_name(&format!("QueueMemberMarker_{}_{}", lane_id, index));
        marker.set_default_color(COLOR_MEMBER_MARKER);
        marker.set_width(MARKER_WIDTH);
        marker.set_closed(true);
        marker.set_z_index(Z_MEMBER_MARKER);
        marker
    }

    fn create_intent_connector(lane_id: EntityId, index: usize) -> Gd<Line2D> {
        let mut connector = Line2D::new_alloc();
        connector.set_name(&format!("QueueIntentConnector_{}_{}", lane_id, index));
        connector.set_default_color(COLOR_CONNECTOR);
        connector.set_width(CONNECTOR_WIDTH);
        connector.set_z_index(Z_CONNECTOR);
        connector
    }

    fn create_intent_marker(lane_id: EntityId, index: usize) -> Gd<Line2D> {
        let mut marker = Line2D::new_alloc();
        marker.set_name(&format!("QueueIntentMarker_{}_{}", lane_id, index));
        marker.set_default_color(COLOR_INTENT_MARKER);
        marker.set_width(INTENT_MARKER_WIDTH);
        marker.set_closed(true);
        marker.set_z_index(Z_INTENT_MARKER);
        marker
    }

    fn cleanup_unseen(&mut self, seen: &HashSet<EntityId>) {
        self.lanes.retain(|lane, nodes| {
            if seen.contains(lane) {
                return true;
            }

            nodes.path.queue_free();
            nodes.rear_marker.queue_free();
            for mut marker in nodes.member_markers.drain(..) {
                marker.queue_free();
            }
            for mut connector in nodes.intent_connectors.drain(..) {
                connector.queue_free();
            }
            for mut marker in nodes.intent_markers.drain(..) {
                marker.queue_free();
            }
            nodes.container.queue_free();
            false
        });
    }

    fn hide_all(&mut self) {
        for nodes in self.lanes.values_mut() {
            nodes.container.set_visible(false);
            nodes.path.set_visible(false);
            nodes.rear_marker.set_visible(false);
            for marker in nodes.member_markers.iter_mut() {
                marker.set_visible(false);
            }
            for connector in nodes.intent_connectors.iter_mut() {
                connector.set_visible(false);
            }
            for marker in nodes.intent_markers.iter_mut() {
                marker.set_visible(false);
            }
        }
    }
}
