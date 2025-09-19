//! Main simulation engine and event handling

use bevy_ecs::prelude::*;

use crate::{models::*, resources::*, systems::*};

/// Simulation event data structure for external observation
///
/// Placeholder for future event system that will communicate
/// game state changes to external observers or UI systems.
pub struct Event;

/// Simulation state snapshot for debugging and analysis
///
/// Placeholder for future snapshot system that will capture
/// complete simulation state for debugging, replay, or analysis.
pub struct Snapshot;

/// Core simulation engine managing ECS world and system execution
///
/// Coordinates the discrete simulation loop, manages entity-component
/// systems, and provides the primary interface for running the
/// canteen dining simulation.
pub struct Simulation {
    /// ECS world containing all entities, components, and resources
    world: World,
    /// System execution schedule defining update order and dependencies
    schedule: Schedule,
}

impl Simulation {
    /// Create a new simulation instance with the provided model registry
    ///
    /// # Arguments
    /// * `db` - Game model registry containing all static configuration data
    pub fn new(db: GameModelRegistry) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        world.insert_resource(Canteen {
            model: db.canteens.first().unwrap().clone(),
        });
        world.insert_resource(db);
        world.insert_resource(CollisionGridRes::default());

        schedule.add_systems(
            (
                update_collision_grid,
                update_diner_spawner,
                update_diner_states,
                check_day_completion,
            )
                .chain(),
        );

        Self { world, schedule }
    }

    /// Initialize and start a simulation level with the given configuration
    ///
    /// Sets up all level-specific resources including RNG seed, diner providers,
    /// spawning parameters, and static world objects. Must be called before
    /// the first tick() to properly initialize the simulation state.
    pub fn start(&mut self, level: LevelConfig) {
        self.world.insert_resource(GameRng::new(level.seed));
        self.world.insert_resource(DinerProvider {
            model: level.diner_provider.clone(),
        });
        self.world.insert_resource(DinerSpawner {
            model: level.diner_spawner.clone(),
            next_spawn_timer: 0.0,
            spawning_finished: false,
        });
        self.world.insert_resource(DayStatus::default());
        self.world.insert_resource(level);

        // Spawn static objects once at startup
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_static_objects);
        schedule.run(&mut self.world);
    }

    /// Advance the simulation by one time step
    ///
    /// Executes all registered systems in the proper order to update
    /// entity states, handle interactions, and progress the simulation.
    /// This should be called at regular intervals to maintain simulation flow.
    ///
    /// # Arguments
    /// * `dt` - Delta time since last tick (used for time initialization)
    pub fn tick(&mut self, dt: f32) {
        // Check if time resource exists and advance it, or create a new one
        match self.world.get_resource_mut::<Time>() {
            Some(mut time) => {
                time.tick();
            }
            None => {
                // Initialize with dt as tick duration if not exists
                let time = Time::new(dt as f64);
                self.world.insert_resource(time);
            }
        }

        self.schedule.run(&mut self.world);
    }

    /// Retrieve all events that occurred during the last simulation step
    pub fn poll_events(&mut self) -> Vec<Event> {
        vec![] // Placeholder
    }

    /// Create a snapshot of the current simulation state for serialization or debugging
    pub fn snapshot(&self) -> Snapshot {
        Snapshot // Placeholder
    }

    /// Check if the current day is complete (spawning finished and all diners left)
    pub fn is_day_complete(&self) -> bool {
        if let (Some(day_status), Some(spawner)) = (
            self.world.get_resource::<DayStatus>(),
            self.world.get_resource::<DinerSpawner>(),
        ) {
            day_status.current_diner_count == 0 && spawner.spawning_finished
        } else {
            false
        }
    }
}
