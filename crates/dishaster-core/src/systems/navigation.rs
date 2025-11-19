use dishaster_navigation::*;

use super::prelude::*;
use crate::utils::ema_alpha_from_dt_tau;

/// System to update the global collision grid
pub fn build_collision_grid(
    mut nav_grid: ResMut<ResWrapper<NavigationGrid>>,
    query: Query<&CompWrapper<BoxCollider>>,
) {
    log::info!("Rebuilding collision grid...");
    nav_grid
        .rebuild()
        .add_colliders(query.iter().map(|c| &**c))
        .done();
}

/// Rebuild crowd cost field from current diner positions
pub fn update_crowd_field(query: Query<&Movement>, mut grid: ResMut<ResWrapper<NavigationGrid>>) {
    // Per-agent contribution caps / scaling
    const MIN_SIGMA_FACTOR: f32 = 0.15; // sigma = max(min, influence_radius * factor)
    const MAX_TILE_CONTRIB: f32 = 50.0; // cap per-agent contribution to a single tile
    const EPSILON: f32 = 0.0001; // avoid div-by-zero & extreme values

    grid.crowd.clear();

    for movement in query.iter() {
        let center = movement.pos;
        let base_weight =
            (movement.radius.max(0.2) * 0.5) + (movement.current_speed.max(0.0) * 0.1);

        let influence_radius = movement.radius * 5.;
        let r2 = influence_radius.squared();

        // Choose sigma for Gaussian falloff; sigma ~ influence_radius * factor
        // Ensure sigma not too small:
        let sigma = (influence_radius * 0.5).max(influence_radius * MIN_SIGMA_FACTOR);
        let inv_two_sigma2 = 1.0 / (2.0 * sigma * sigma + EPSILON);

        // Compute bounding tile region (integer tile coords)
        let tile_radius = world_to_tile_dist(influence_radius, grid.cell_size()).ceil() as i32;
        let center_tile = grid.world_to_igrid(center);

        for tx in (center_tile.x - tile_radius)..=(center_tile.x + tile_radius) {
            for ty in (center_tile.y - tile_radius)..=(center_tile.y + tile_radius) {
                let Some(tile_idx) = grid.bound_tile(ivec2(tx, ty)) else {
                    continue;
                };
                // compute squared distance from agent center to tile center (no sqrt)
                let tile_world = grid.tile_to_world(tile_idx);
                let d2 = center.distance_squared(tile_world);

                // early skip if outside circle
                if d2 > r2 {
                    continue;
                }

                // Gaussian falloff: contrib = base_weight * exp(-d2 / (2*sigma^2))
                // Scale magnitude with movement.radius (or explicit weight)
                let gauss = (-d2 * inv_two_sigma2).exp();
                // Increased scale factor from 10.0 to 15.0 for stronger crowd avoidance
                let mut extra = base_weight * gauss * 15.0; // scale factor for game tuning

                // Bound contribution to avoid catastrophic values near d2->0
                if extra.is_nan() || extra.is_infinite() || extra > MAX_TILE_CONTRIB {
                    extra = MAX_TILE_CONTRIB;
                }
                grid.crowd.add_cost(tile_idx, extra);
            }
        }
    }

    // In the current implementation, the typical max cost is around 4.
}

/// Update movement speeds dynamically based on multiple factors.
///
/// **Algorithm Overview:**
/// This system implements a multiplicative speed model where an agent's speed is affected by:
/// 1. Personal factors: base speed × mobility (physical capability)
/// 2. Psychological factor: urgency (impatience/hurry level)
/// 3. Environmental factor: crowd density (slower in crowded areas)
/// 4. Physical load: carry weight (slower when carrying heavy trays)
///
/// **Formula:**
/// ```ignore
/// target_speed = clamp(
///     base_speed × mobility × (1 + u_gain×urgency)
///     × 1/(1 + crowd_sensitivity×density)
///     × 1/(1 + carry_sensitivity×weight),
///     min_speed, max_speed
/// )
/// ```
///
/// **Performance Optimization:**
/// - Updates are periodic (every 0.3s), not every frame, to reduce CPU cost
/// - Each agent updates at slightly different times (natural jitter from simulation timing)
/// - Uses exponential moving average (EMA) for smooth transitions, avoiding jarring speed changes
pub fn update_movement_speeds(
    mut query: Query<(&mut Movement, Option<&DinerState>)>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
    time: Res<Time>,
) {
    // Urgency gain: urgency=1.0 increases speed by 35%
    // Example: impatient diner at urgency=1.0 moves 35% faster than baseline
    const U_GAIN: f32 = 0.35;

    // Crowd sensitivity: controls how much crowd slows agents down
    // Increased from 2.5 to 3.0 for stronger crowd avoidance
    // At density=0.4, crowd_factor = 1/(1+3.0×0.4) = 0.45 (55% speed reduction)
    const CROWD_SENSITIVITY: f32 = 3.0;

    // Carry sensitivity: weight penalty coefficient
    // At 2kg, carry_factor = 1/(1+0.25×2) = 0.67 (33% slower)
    const CARRY_SENSITIVITY: f32 = 0.5;

    // Speed limits to prevent unrealistic movement
    const MIN_SPEED: f32 = 0.3; // m/s - minimum crawl speed in extreme congestion
    const MAX_SPEED: f32 = 2.0; // m/s - maximum sprint speed

    // Periodic update interval: recalculate target speed every 0.3s
    // Reduces computation and prevents jitter from rapid crowd changes
    const UPDATE_INTERVAL: f32 = 0.3; // seconds

    // EMA time constant: controls smoothing strength
    // Smaller tau = faster response, larger tau = smoother but slower adaptation
    const TAU: f32 = 0.3; // seconds

    // Calculate EMA smoothing factor: alpha = dt/tau (clamped to prevent overshoot)
    // This makes speed changes feel natural rather than instantaneous
    let dt = time.tick_duration as f32;
    let current_time = time.current_time as f32;
    let alpha = ema_alpha_from_dt_tau(dt, TAU); // EMA: new_value += (target - new_value) × alpha

    for (mut movement, diner_state) in query.iter_mut() {
        // ===== Staggered Update Strategy =====
        // Skip agents whose update interval hasn't elapsed yet
        // This distributes CPU load over multiple frames and prevents synchronized jitter
        if current_time - movement.last_speed_update < UPDATE_INTERVAL {
            continue;
        }
        movement.last_speed_update = current_time;

        // Staff (no DinerState) use simple speed calculation without crowd/carry factors
        if diner_state.is_none() {
            let target_speed = movement.walking_speed.clamp(MIN_SPEED, MAX_SPEED);
            movement.current_speed += (target_speed - movement.current_speed) * alpha;
            continue;
        }

        // ===== Factor 1: Base Speed × Mobility =====
        // base_speed: agent's natural walking speed (e.g., 1.3 m/s for average adult)
        // mobility: physical capability modifier (0.7-1.2), accounts for age/fitness/disability
        let base_speed = movement.walking_speed;
        let mobility = movement.speed_factor;

        // ===== Factor 2: Urgency Boost =====
        // Maps psychological impatience [0,1] to speed multiplier [1.0, 1.35]
        // urgency=0 (relaxed) → no change, urgency=1 (very hurried) → +35% speed
        let urgency = movement.impatience.clamp(0.0, 1.0);
        let urgency_factor = 1.0 + U_GAIN * urgency;

        // ===== Factor 3: Crowd Density Penalty =====
        // Sample normalized crowd density [0,1] from grid at agent's position
        // Uses reciprocal formula to ensure smooth slowdown without division by zero
        // density=0 → factor=1.0 (no penalty), density=0.5 → factor=0.29 (71% slower)
        let tile = nav_grid.world_to_grid(movement.pos);
        let crowd_density = nav_grid.crowd.sample_normalized(tile);
        let crowd_factor = 1.0 / (1.0 + CROWD_SENSITIVITY * crowd_density);

        // ===== Factor 4: Carry Weight Penalty =====
        // Calculate total weight from tray + served dishes (only for diners)
        // Staff and other agents have weight=0 (no penalty)
        // Uses same reciprocal formula as crowd for consistency
        // weight=0 → factor=1.0, weight=2kg → factor=0.67 (33% slower)
        let carry_weight = diner_state
            .map(|state| state.total_carry_weight())
            .unwrap_or(0.0);
        let carry_factor = 1.0 / (1.0 + CARRY_SENSITIVITY * carry_weight);

        // ===== Compute Target Speed =====
        // Multiply all factors together and clamp to realistic bounds
        // Example: base=1.3, mobility=1.0, urgency=0.5 → urgency_factor=1.175
        //          crowd_density=0.3 → crowd_factor=0.57, carry=1kg → carry_factor=0.8
        //          target = 1.3 × 1.0 × 1.175 × 0.57 × 0.8 ≈ 0.70 m/s
        let target_speed = (base_speed * mobility * urgency_factor * crowd_factor * carry_factor)
            .clamp(MIN_SPEED, MAX_SPEED);

        // ===== EMA Smoothing =====
        // Gradually adjust current_speed toward target_speed using exponential moving average
        // This prevents sudden jerky movements when conditions change
        // Formula: current += (target - current) × alpha
        // With tau=0.3s, reaches ~95% of target within 1 second
        movement.current_speed += (target_speed - movement.current_speed) * alpha;
    }
}

impl Movement {
    /// Request a path to the specified target (point or rect).
    pub fn request_path_any(&mut self, target: PathTarget) {
        log::trace!(target: "navigation", "Path requested to {target:.2}");

        self.pending_target = Some(target);
        self.target_reached = false;
    }

    /// Request a path to the specified target position.
    pub fn request_path(&mut self, target: Vec2) {
        log::trace!(target: "navigation", "Path requested to {target:.2}");

        self.pending_target = Some(PathTarget::Point(target));
        self.target_reached = false;
    }

    /// Request a path to the specified target position.
    pub fn request_path_to_rect(&mut self, target: Rect) {
        log::trace!(target: "navigation", "Path requested to {target:?}");

        self.pending_target = Some(PathTarget::Rect(target));
        self.target_reached = false;
    }

    /// Computes a new path to the specified target, updating the target position and path.
    /// If no valid path is found, the path is cleared.
    fn compute_new_path(&mut self, target: PathTarget, nav_grid: &NavigationGrid) {
        // Precheck
        match target {
            PathTarget::Point(target) => {
                if !nav_grid.is_pos_traversable(target, self.radius) {
                    log::debug!(target: "navigation", "Path target {:.2} not traversable", target);
                    self.path.clear();
                    return;
                }
                if self.pos.close_to(target, 0.1) {
                    self.set_reached();
                    return;
                }
            }
            PathTarget::Rect(target) => {
                if target.inflate(0.15).contains(self.pos) {
                    log::debug!(target: "navigation", "Pos {} already inside target rect {:?}", self.pos, target);
                    self.set_reached();
                    return;
                }
            }
        }

        if let Some(path) = find_path(PathRequest {
            start: self.pos,
            target,
            radius: self.radius,
            impatience: self.impatience,
            grid: nav_grid,
        }) {
            log::trace!(
                target: "navigation",
                "Path from {:.2} to {:.2} found with {} waypoints",
                self.pos,
                target,
                path.len(),
            );

            self.current_target = Some(target);
            self.target_reached = false;
            self.path = path;
        } else {
            log::debug!(target: "navigation", "Path from {:.2} to {:.2} not found", self.pos, target);

            self.path.clear();
        }
    }

    fn set_reached(&mut self) {
        self.path.clear();
        self.target_reached = true;
        self.current_target = None;
        self.velocity = Vec2::ZERO;
    }
}

/// Process pending path requests for agents.
pub fn run_path_requests(
    mut query: Query<&mut Movement>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
    time: Res<Time>,
) {
    const PATH_COOLDOWN_TICKS: Tick = 60;

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
    mut query: Query<(Entity, &mut Movement)>,
    nav_grid: Res<ResWrapper<NavigationGrid>>,
    time: Res<Time>,
    mut rng: ResMut<NavigationRng>,
) {
    let dt = time.tick_duration as f32;
    let waypoint_eps = PATH_WAYPOINT_EPS;
    let accel = 4.0; // Steering acceleration factor
    let stop_eps = 0.01; // Distance to target to stop moving

    let get_next_velocity = |movement: &Movement| -> Vec2 {
        let Some(next_pos) = movement.path.next() else {
            return Vec2::ZERO; // No path, no movement
        };

        let displacement = next_pos - movement.pos;

        // Immediate stop if very close to target
        if displacement.length_squared() < stop_eps.squared() {
            return Vec2::ZERO;
        }

        // Determine desired velocity using dynamic current_speed
        let dir = if let Some(next) = movement.path.next() {
            (next - movement.pos).normalize_or_zero()
        } else {
            displacement.normalize_or_zero()
        };
        let max_speed = movement.current_speed; // Use dynamically calculated speed
        let desired_velocity = dir * max_speed;

        desired_velocity.clamp_length_max(max_speed)
    };

    // ===== Build Avoidance Agent List =====
    // **Algorithm:**
    // 1. Iterate through all agents
    // 2. Skip agents with empty paths (they're done moving)
    // 3. Build nav_agents list with only active movers
    // 4. Track entity IDs to map results back later
    let mut nav_agents = Vec::new();
    let mut entities = Vec::new(); // Maps nav_agent index -> entity

    for (entity, m) in query.iter() {
        if m.path.is_empty() {
            // Skip agents without paths - they shouldn't participate in avoidance
            // This is the key fix for the head-on circling bug
            continue;
        }

        // Dynamic radius adjustment based on crowd density
        // In crowded areas, agents need more personal space to avoid overlap
        let tile = nav_grid.world_to_grid(m.pos);
        let crowd_density = nav_grid.crowd.sample_normalized(tile);
        // Expand radius by up to 30% in high-density areas
        let radius_multiplier = 1.0 + (crowd_density * 0.3);
        let effective_radius = m.radius * radius_multiplier;

        nav_agents.push(dishaster_navigation::Agent {
            position: m.pos,
            velocity: get_next_velocity(m),
            goal: m.path.next().unwrap_or(m.pos),
            radius: effective_radius,
            max_velocity: m.current_speed, // Use dynamically calculated speed
            avoidance_responsibility: m.avoidance_responsibility,
        });
        entities.push(entity);
    }

    let new_velocities = nav_grid.get_updated_velocities(&nav_agents, dt);

    // Apply new velocities only to agents with active paths
    for (nav_idx, entity) in entities.iter().enumerate() {
        let Ok((_entity, mut movement)) = query.get_mut(*entity) else {
            continue;
        };
        let velocity = new_velocities[nav_idx];

        // Steering: adjust velocity towards desired
        let steer = velocity - movement.velocity;
        let velocity = movement.velocity + steer * accel * dt;

        // Update position by velocity
        movement.velocity = velocity;
        movement.pos += velocity * dt;
        movement.pos = nav_grid.clamp(movement.pos); // Keep within bounds

        // Update next_waypoint
        if let Some(next) = movement.path.next()
            && movement.pos.close_to(next, waypoint_eps)
        {
            movement.path.pop();
            if movement.path.is_empty() {
                movement.velocity = Vec2::ZERO;
                movement.target_reached = true;
                movement.current_target = None;
            }
        }

        // Randomly re-find path due to crowd update
        const PATH_COOLDOWN_TICKS: Tick = 300;
        if movement.pending_target.is_none()
            && let Some(target) = movement.current_target
            && time.current_tick - movement.last_path_tick >= PATH_COOLDOWN_TICKS
            && rng.random_bool(0.01)
        {
            movement.request_path_any(target);
            log::debug!("Randomly re-requesting path to {target:.2}",);
        }
    }
}

/// Keep display Transform in sync with Movement position (x,y), preserving z.
pub fn sync_transform_with_movement(mut query: Query<(&Movement, &mut Transform)>) {
    for (movement, mut transform) in query.iter_mut() {
        transform.position = movement.pos.extend(0.0);
    }
}
