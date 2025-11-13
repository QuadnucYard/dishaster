use dodgy::AvoidanceOptions;

use crate::{NavigationGrid, prelude::*};

pub(crate) trait FromDodgy<T> {
    fn from_dodgy(value: T) -> Self;
}

pub(crate) trait IntoDodgy<T> {
    fn into_dodgy(self) -> T;
}

impl<T> IntoDodgy<T> for T
where
    T: FromDodgy<T>,
{
    fn into_dodgy(self) -> T {
        T::from_dodgy(self)
    }
}

impl FromDodgy<dodgy::Vec2> for Vec2 {
    fn from_dodgy(value: dodgy::Vec2) -> Self {
        Vec2::new(value.x, value.y)
    }
}

impl IntoDodgy<dodgy::Vec2> for Vec2 {
    fn into_dodgy(self) -> dodgy::Vec2 {
        dodgy::Vec2::new(self.x, self.y)
    }
}

/// A single agent in the simulation. This is an adapter for [`dodgy::Agent`], due to the
/// inconsistent version of [`glam::Vec2`].
#[derive(Clone, PartialEq, Debug)]
pub struct Agent {
    /// The position of the agent.
    pub position: Vec2,
    /// The current velocity of the agent.
    pub velocity: Vec2,
    /// The goal position of the agent. The agent will attempt to move towards this
    pub goal: Vec2,
    /// The radius of the agent. Agents will use this to avoid bumping into each
    /// other.
    pub radius: f32,
    /// The maximum velocity the agent is allowed to move at.
    pub max_velocity: f32,

    /// The amount of responsibility an agent has to avoid other agents. The
    /// amount of avoidance between two agents is then dependent on the ratio of
    /// the responsibility between the agents. Note this does not affect
    /// avoidance of obstacles.
    pub avoidance_responsibility: f32,
}

impl From<Agent> for dodgy::Agent {
    fn from(val: Agent) -> Self {
        dodgy::Agent {
            position: dodgy::Vec2::new(val.position.x, val.position.y),
            velocity: dodgy::Vec2::new(val.velocity.x, val.velocity.y),
            radius: val.radius,
            max_velocity: val.max_velocity,
            avoidance_responsibility: val.avoidance_responsibility,
        }
    }
}

impl NavigationGrid {
    /// Given a list of agents, compute their new velocities after applying
    /// collision avoidance.
    pub fn get_updated_velocities(&self, agents: &[Agent], time_step: f32) -> Vec<Vec2> {
        let dodgy_agents: Vec<dodgy::Agent> = agents.iter().cloned().map(Into::into).collect();

        let avoidance_options = AvoidanceOptions {
            obstacle_margin: 0.1,
            time_horizon: 3.0,
            obstacle_time_horizon: 1.0,
        };

        let mut new_velocities = Vec::with_capacity(dodgy_agents.len());

        const NEIGHBOR_RADIUS: f32 = 3.0;
        const NEIGHBOR_RADIUS_SQ: f32 = NEIGHBOR_RADIUS * NEIGHBOR_RADIUS;
        const MAX_NEIGHBORS: usize = 8;
        let mut neighbors = Vec::with_capacity(MAX_NEIGHBORS); // reused
        let mut nearby_obstacles = Vec::new(); // reused

        for (i, agent) in dodgy_agents.iter().enumerate() {
            // Gather only nearby agents so the solver does not consider the whole crowd.
            neighbors.clear();
            for (j, candidate) in dodgy_agents.iter().enumerate() {
                if j != i
                    && candidate.position.distance_squared(agent.position) <= NEIGHBOR_RADIUS_SQ
                {
                    neighbors.push(candidate);
                    if neighbors.len() == MAX_NEIGHBORS {
                        break;
                    }
                }
            }

            // ===== Goal Approach Speed Reduction (Prevents Overshooting) =====
            // Calculate vector and distance to goal
            let to_goal = agents[i].goal.into_dodgy() - agent.position;
            let distance_to_goal = to_goal.length();

            // **Algorithm: Linear Speed Ramp-Down Near Goal**
            // Problem: Agents moving at full speed toward goals can overshoot and circle
            // Solution: Gradually reduce speed as agent approaches goal
            //
            // Slowdown zone: Starts at (agent.radius × 2) from goal
            // - Outside zone: speed_factor = 1.0 (full speed)
            // - Inside zone: speed_factor = distance/slowdown_start (linear decay)
            // - Very close: minimum 10% speed to avoid complete stalls
            //
            // Example: agent.radius=0.3m, slowdown_start=0.6m
            //   distance=0.6m → factor=1.0 (full speed)
            //   distance=0.3m → factor=0.5 (half speed)
            //   distance=0.06m → factor=0.1 (10% speed, minimum)
            //
            // This enables smooth arrival without oscillation or circling
            const SLOWDOWN_DISTANCE_FACTOR: f32 = 2.0;
            let slowdown_start = agent.radius * SLOWDOWN_DISTANCE_FACTOR;
            let speed_factor = if distance_to_goal < slowdown_start {
                (distance_to_goal / slowdown_start).max(0.1) // Min 10% speed near goal
            } else {
                1.0
            };

            // Apply speed reduction to preferred velocity
            let preferred_velocity =
                to_goal.normalize_or_zero() * (agent.max_velocity * speed_factor);

            nearby_obstacles.clear();
            for obstacle in self.obstacles() {
                if let dodgy::Obstacle::Closed { vertices } = obstacle
                    && is_obstacle_within_radius(agent.position, vertices, NEIGHBOR_RADIUS)
                {
                    nearby_obstacles.push(obstacle);
                }
            }

            let avoidance_velocity = agent.compute_avoiding_velocity(
                &neighbors,
                &nearby_obstacles,
                preferred_velocity,
                time_step,
                &avoidance_options,
            );
            new_velocities.push(Vec2::from_dodgy(avoidance_velocity));
        }

        new_velocities
    }
}

fn is_obstacle_within_radius(pos: dodgy::Vec2, vertices: &[dodgy::Vec2], radius: f32) -> bool {
    if vertices.is_empty() {
        return false;
    }

    let mut min_x = vertices[0].x;
    let mut max_x = vertices[0].x;
    let mut min_y = vertices[0].y;
    let mut max_y = vertices[0].y;

    // Determine the obstacle's axis-aligned bounding box.
    for vertex in &vertices[1..] {
        min_x = min_x.min(vertex.x);
        max_x = max_x.max(vertex.x);
        min_y = min_y.min(vertex.y);
        max_y = max_y.max(vertex.y);
    }

    let dx = if pos.x < min_x {
        min_x - pos.x
    } else if pos.x > max_x {
        pos.x - max_x
    } else {
        0.0
    };

    let dy = if pos.y < min_y {
        min_y - pos.y
    } else if pos.y > max_y {
        pos.y - max_y
    } else {
        0.0
    };

    dx + dy <= radius
}
