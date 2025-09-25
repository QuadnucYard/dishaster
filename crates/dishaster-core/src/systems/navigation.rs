use dishaster_navigation::{CollisionEntity, world_to_tile_dist};
use dishrupt_core::display::Transform;

use crate::{components::*, constants::*, models::*, prelude::*, resources::*};

/// System to update the global collision grid
pub fn update_collision_grid(
    mut collision_grid: ResMut<CollisionGridRes>,
    query: Query<(Entity, &BoxCollider)>,
) {
    collision_grid.update(
        query
            .iter()
            .map(|(e, c)| (CollisionEntity(e.to_bits()), &**c)),
    );
}

/// Rebuild crowd cost field from current diner positions
pub fn update_crowd_field(
    mut field: ResMut<CrowdFieldRes>,
    diners: Query<(&Movement, &DinerModel)>,
    grid: Res<CollisionGridRes>,
) {
    // Initialize field with current grid cell size
    field.set_cell_size(grid.cell_size());
    field.clear();

    // Parameters: influence radius and decay scale based on diner attributes (e.g., patience)
    for (movement, model) in diners.iter() {
        let center = movement.pos;
        let patience = model.attributes.patience.max(0.1);
        // Larger patience -> prefers more space; increase radius and weight
        let influence_radius = 2.0 + 4.0 * patience; // meters
        let max_extra = 5.0 * patience; // peak extra cost at center

        // Compute bounding tiles
        let tile_radius = world_to_tile_dist(influence_radius, grid.cell_size()).ceil() as i32;
        let center_tile = grid.world_to_grid(center);
        for dx in -tile_radius..=tile_radius {
            for dy in -tile_radius..=tile_radius {
                let t = center_tile + IVec2::new(dx, dy);
                let world = grid.tile_to_world(t);
                let d = center.distance(world);
                if d <= influence_radius {
                    // Smooth decay: extra = max_extra * (1 - (d/r)^2)
                    let r = influence_radius.max(0.001);
                    let extra = max_extra * (1.0f32 - (d / r).powi(2)).max(0.0f32);
                    field.add_cost(t, extra);
                }
            }
        }
    }
}

/// Move agents along their planned paths with smooth steering.
///
/// This system advances Movement.pos using velocity-based steering for smoother
/// movement and turns. Agents accelerate towards desired velocity, clamped to max speed.
pub fn update_agent_movement(
    time: Res<Time>,
    canteen: Res<Canteen>,
    collision_grid: Res<CollisionGridRes>,
    mut query: Query<(Entity, &mut Movement, &mut BoxCollider)>,
) {
    let dt = time.tick_duration as f32;
    let max_speed = DINER_SPEED_MPS;
    let waypoint_eps = PATH_WAYPOINT_EPS;
    let accel = 5.0; // Steering acceleration factor
    let stop_eps = 0.5; // Distance to target to stop moving

    // Separation tuning
    let sep_gain = 2.0; // strength multiplier for repulsion

    for (entity, mut movement, mut collider) in query.iter_mut() {
        movement.last_pos = movement.pos;
        let current_pos = movement.pos;
        let self_entity = CollisionEntity(entity.to_bits());

        // Determine desired velocity
        let next_pos = movement.path.first().copied();
        let dist_to_target = (movement.target_pos - current_pos).length();
        let desired_velocity = if let Some(next) = next_pos {
            let dir = (next - current_pos).normalize_or_zero();
            dir * max_speed
        } else if dist_to_target < stop_eps {
            Vec2::ZERO
        } else {
            let dir = (movement.target_pos - current_pos).normalize_or_zero();
            dir * max_speed
        };

        // Steering: adjust velocity towards desired
        let steer = desired_velocity - movement.velocity;
        movement.velocity += steer * accel * dt;
        movement.velocity = movement.velocity.clamp_length_max(max_speed);

        // Immediate stop if very close to target
        let dist_to_target = (movement.target_pos - movement.pos).length();
        if dist_to_target < stop_eps {
            movement.velocity = Vec2::ZERO;
        }

        // Update position by velocity
        let mut new_pos = movement.pos + movement.velocity * dt;

        // Apply local separation to avoid overlaps with nearby agents
        let search_radius = (collider.size.x.max(collider.size.y)) * 2.0;
        let mut separation = Vec2::ZERO;
        for other in collision_grid.find_nearby_entities(new_pos, search_radius) {
            if other == self_entity {
                continue;
            }
            if let Some(other_col) = collision_grid.collider(other) {
                let delta = new_pos - other_col.center;
                let dist = delta.length();
                if dist <= 0.0001 {
                    continue;
                }
                let desired = (collider.size.x + other_col.size.x) * 0.5;
                if dist < desired {
                    let push_dir = delta / dist;
                    let strength = (desired - dist) / desired; // 0..1
                    separation += push_dir * strength;
                }
            }
        }
        if separation != Vec2::ZERO {
            new_pos += separation * sep_gain * dt * DINER_SPEED_MPS;
        }

        // Clamp within canteen bounds
        let ww = canteen.model.width;
        let wh = canteen.model.height;
        let clamped = Vec2::new(new_pos.x.clamp(0.0, ww), new_pos.y.clamp(0.0, wh));
        if clamped != new_pos {
            log::trace!(
                target: "nav",
                "move: clamped from ({:.2},{:.2}) to ({:.2},{:.2}) within bounds [0,{:.2}]x[0,{:.2}]",
                new_pos.x,
                new_pos.y,
                clamped.x,
                clamped.y,
                ww,
                wh
            );
            new_pos = clamped;
            // Reset velocity to prevent bouncing
            movement.velocity = Vec2::ZERO;
        }

        movement.pos = new_pos;

        // Update next_waypoint
        if let Some(next) = movement.path.first().cloned() {
            movement.next_waypoint = next;
            let dist = movement.pos.distance(next);
            if dist < waypoint_eps {
                movement.path.remove(0);
                log::trace!(target: "nav", "move: pop waypoint ({:.2},{:.2})", next.x, next.y);
            }
        } else {
            movement.next_waypoint = movement.target_pos;
        }

        // Sync collider center to movement position
        collider.center = movement.pos;
    }
}

/// Keep display Transform in sync with Movement position (x,y), preserving z.
pub fn sync_transform_with_movement(mut query: Query<(&Movement, &mut Transform)>) {
    for (movement, mut transform) in query.iter_mut() {
        transform.position.x = movement.pos.x;
        transform.position.y = movement.pos.y;
        // z remains unchanged (layering is handled by display logic)
    }
}
