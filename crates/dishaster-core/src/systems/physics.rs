use dishrupt_core::display::Transform;

use crate::{components::*, constants::*, prelude::*, resources::*};

/// System to update the global collision grid
pub fn update_collision_grid(
    mut collision_grid: ResMut<CollisionGridRes>,
    query: Query<(Entity, &BoxCollider)>,
) {
    collision_grid.update(&query);
}

/// Move agents along their planned paths with smooth steering.
///
/// This system advances Movement.pos using velocity-based steering for smoother
/// movement and turns. Agents accelerate towards desired velocity, clamped to max speed.
pub fn update_agent_movement(
    time: Res<Time>,
    canteen: Res<Canteen>,
    mut query: Query<(&mut Movement, &mut BoxCollider)>,
) {
    let dt = time.tick_duration as f32;
    let max_speed = DINER_SPEED_MPS;
    let waypoint_eps = PATH_WAYPOINT_EPS;
    let accel = 5.0; // Steering acceleration factor
    let stop_eps = 0.5; // Distance to target to stop moving

    for (mut movement, mut collider) in query.iter_mut() {
        movement.last_pos = movement.pos;
        let current_pos = movement.pos;

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

        // Update position
        let mut new_pos = movement.pos + movement.velocity * dt;

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
