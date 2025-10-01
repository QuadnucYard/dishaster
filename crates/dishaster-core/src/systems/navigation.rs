use dishaster_navigation::{NavigationGrid, PathRequest, find_path, world_to_tile_dist};
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
pub fn update_crowd_field(query: Query<&Movement>, mut grid: ResMut<ResWrapper<NavigationGrid>>) {
    grid.crowd.clear();

    for movement in query.iter() {
        let center = movement.pos;
        let influence_radius = movement.radius * 5.0;
        let r2 = influence_radius.squared();
        let max_extra = movement.radius * 5.0; // todo: improve model

        // Compute bounding tiles
        let tile_radius = world_to_tile_dist(influence_radius, grid.cell_size()).ceil() as i32;
        let center_tile = grid.world_to_igrid(center);
        for dx in -tile_radius..=tile_radius {
            for dy in -tile_radius..=tile_radius {
                let Some(t) = grid.bound_tile(center_tile + IVec2::new(dx, dy)) else {
                    continue;
                };
                let d2 = center.distance_squared(grid.tile_to_world(t));
                if d2 <= influence_radius.squared() {
                    // Smooth decay: extra = m * (r / d)^2
                    let extra = max_extra * (r2 / (d2 + 1.0));
                    grid.crowd.add_cost(t, extra);
                }
            }
        }
    }
}

impl Movement {
    /// Request a path to the specified target position.
    pub fn request_path(&mut self, target: Vec2) {
        self.pending_target = Some(target);
    }

    /// Computes a new path to the specified target, updating the target position and path.
    /// If no valid path is found, the path is cleared.
    fn compute_new_path(&mut self, target: Vec2, nav_grid: &NavigationGrid) {
        if let Some(path) = find_path(PathRequest {
            start: self.pos,
            end: target,
            radius: self.radius,
            impatience: self.impatience,
            grid: nav_grid,
        }) {
            self.path = path;
        } else {
            self.path.clear();
        }
    }
}

/// Process pending path requests for agents.
pub fn run_path_requests(
    mut query: Query<&mut Movement>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
    time: Res<Time>,
) {
    const PATH_COOLDOWN_TICKS: u32 = 60;

    for mut movement in query.iter_mut() {
        if let Some(target) = movement.pending_target {
            if time.current_tick - movement.last_path_tick < PATH_COOLDOWN_TICKS {
                continue;
            }
            movement.compute_new_path(target, &nav_grid);
            movement.pending_target = None;
            movement.last_path_tick = time.current_tick;
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
        let Some(next_pos) = movement.path.next() else {
            return Vec2::ZERO; // No path, no movement
        };

        let displacement = next_pos - movement.pos;

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
            goal: m.path.next().unwrap_or(m.pos),
            radius: m.radius,
            max_velocity: m.walking_speed * m.speed_factor,
            avoidance_responsibility: 1.0, // TODO
        })
        .collect::<Vec<_>>();

    let new_velocities = nav_grid.get_updated_velocities(&nav_agents, dt);

    // Apply new velocities and update positions
    for (mut movement, velocity) in query.iter_mut().zip(new_velocities) {
        if movement.path.is_empty() {
            movement.velocity = Vec2::ZERO;
            continue;
        }

        // Update position by velocity
        movement.velocity = velocity;
        movement.pos += velocity * dt;

        // Update next_waypoint
        if let Some(next) = movement.path.next()
            && movement.pos.close_to(next, waypoint_eps)
        {
            movement.path.pop();
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
