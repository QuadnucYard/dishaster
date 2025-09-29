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

        for i in 0..dodgy_agents.len() {
            let neighbours = dodgy_agents[..i]
                .iter()
                .chain(dodgy_agents[(i + 1)..].iter())
                .collect::<Vec<_>>();
            let nearby_obstacles = self.obstacles().iter().collect::<Vec<_>>();

            let preferred_velocity = (agents[i].goal.into_dodgy() - dodgy_agents[i].position)
                .normalize_or_zero()
                * dodgy_agents[i].max_velocity;

            let avoidance_velocity = dodgy_agents[i].compute_avoiding_velocity(
                &neighbours,
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
