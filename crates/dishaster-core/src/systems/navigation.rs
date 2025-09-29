use dishaster_navigation::{NavigationGrid, world_to_tile_dist};
use dishrupt_core::display::Transform;

use crate::{components::*, constants::*, prelude::*, resources::*};

/// System to update the global collision grid
pub fn build_collision_grid(
    mut nav_grid: ResMut<ResWrapper<NavigationGrid>>,
    query: Query<&BoxCollider>,
) {
    log::info!("Rebuilding collision grid...");
    nav_grid.update(query.iter().map(|c| &**c));
}

/// Rebuild crowd cost field from current diner positions
pub fn update_crowd_field(
    diners: Query<(&Movement, &DinerModelComp)>,
    mut grid: ResMut<ResWrapper<NavigationGrid>>,
) {
    grid.crowd.clear();

    // Parameters: influence radius and decay scale based on diner attributes (e.g., patience)
    for (movement, model) in diners.iter() {
        let center = movement.pos;
        let patience = model.attributes.patience.max(0.1);
        // Larger patience -> prefers more space; increase radius and weight
        let influence_radius = 2.0 + 4.0 * patience; // meters
        let max_extra = 5.0 * patience; // peak extra cost at center

        // Compute bounding tiles
        // TODO: here we borrows grid from collision grid, which may be buggy when they have different grid sizes.
        let tile_radius = world_to_tile_dist(influence_radius, grid.cell_size()).ceil() as i32;
        let center_tile = grid.world_to_igrid(center);
        for dx in -tile_radius..=tile_radius {
            for dy in -tile_radius..=tile_radius {
                let Some(t) = grid.bound_tile(center_tile + IVec2::new(dx, dy)) else {
                    continue;
                };
                let d2 = center.distance_squared(grid.tile_to_world(t));
                if d2 <= influence_radius.squared() {
                    // Smooth decay: extra = max_extra * (1 - (d/r)^2)
                    let r = influence_radius.max(0.001);
                    let extra = max_extra * (1.0f32 - d2 / r.squared()).max(0.0f32);
                    grid.crowd.add_cost(t, extra);
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
    _canteen: Res<Canteen>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
    mut query: Query<&mut Movement>,
) {
    let dt = time.tick_duration as f32;
    let waypoint_eps = PATH_WAYPOINT_EPS;
    let accel = 5.0; // Steering acceleration factor
    let stop_eps = 0.5; // Distance to target to stop moving

    let get_next_velocity = |movement: &Movement| -> Vec2 {
        let displacement = movement.target_pos - movement.pos;

        // Immediate stop if very close to target
        if displacement.length_squared() < stop_eps.squared() {
            return Vec2::ZERO;
        }

        // Determine desired velocity
        let dir = if let Some(next) = movement.path.next() {
            (next - movement.pos).normalize_or_zero()
        } else {
            displacement.normalize_or_zero()
        };
        let max_speed = movement.walking_speed * movement.speed_factor;
        let desired_velocity = dir * max_speed;

        // Steering: adjust velocity towards desired
        let steer = desired_velocity - movement.velocity;
        let velocity = movement.velocity + steer * accel * dt;

        velocity.clamp_length_max(max_speed)
    };

    let nav_agents = query
        .iter()
        .map(|m| dishaster_navigation::Agent {
            position: m.pos,
            velocity: get_next_velocity(m),
            goal: m.next_waypoint,
            radius: m.radius,
            max_velocity: m.walking_speed * m.speed_factor,
            avoidance_responsibility: 1.0, // TODO
        })
        .collect::<Vec<_>>();

    let new_velocities = nav_grid.get_updated_velocities(&nav_agents, dt);

    // Apply new velocities and update positions
    for (mut movement, velocity) in query.iter_mut().zip(new_velocities) {
        // Update position by velocity
        movement.velocity = velocity;
        movement.pos += velocity * dt;

        // Update next_waypoint
        if let Some(next) = movement.path.next() {
            movement.next_waypoint = next;
            if movement.pos.close_to(next, waypoint_eps) {
                movement.path.pop();
            }
        } else {
            movement.next_waypoint = movement.target_pos;
        }
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
