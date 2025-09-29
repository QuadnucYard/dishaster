//! Main simulation engine and event handling

use std::{cmp::Ordering, num::NonZero, sync::Arc};

use dishaster_navigation::*;
use dishrupt_core::{EntityId, display::*};

use crate::{models::*, prelude::*, resources::*, snapshots::*, systems::*};

/// Simulation event data structure for external observation
///
/// Placeholder for future event system that will communicate
/// game state changes to external observers or UI systems.
pub struct Event;

/// Core simulation engine managing ECS world and system execution
///
/// Coordinates the discrete simulation loop, manages entity-component
/// systems, and provides the primary interface for running the
/// canteen dining simulation.
pub struct Simulation {
    /// ECS world containing all entities, components, and resources
    pub(crate) world: World,
    /// System execution schedule defining update order and dependencies
    schedule: Schedule,
    /// Debug feature configuration for snapshot export.
    pub(crate) debug_flags: DebugFeatureFlags,
}

impl Simulation {
    /// Create a new simulation instance with the provided model registry
    ///
    /// # Arguments
    /// * `db` - Game model registry containing all static configuration data
    pub fn new(db: Arc<GameModelRegistry>) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        let root_entity = world.spawn(Transform::default()).id();
        world.insert_resource(DisplayRoot(root_entity));

        world.insert_resource(db.into_res());

        schedule.add_systems(
            (
                // Recompute soft crowd costs for pathfinding
                update_crowd_field,
                // Spawn logic may add diners
                update_diner_spawner,
                // Agents decide targets and compute paths
                update_diner_states,
                // Update queue ordering and slot targets before movement
                update_window_queues,
                // Move agents along paths
                update_agent_movement,
                // Sync visuals to movement positions
                sync_transform_with_movement,
                // Despawn agents who reached exits and update counts
                despawn_leaving_diners,
                check_day_completion,
            )
                .chain(),
        );

        Self {
            world,
            schedule,
            debug_flags: DebugFeatureFlags::all(),
        }
    }

    /// Update the debug feature configuration for snapshot export.
    pub fn set_debug_flags(&mut self, flags: DebugFeatureFlags) {
        self.debug_flags = flags;
    }

    /// Retrieve the current debug feature configuration.
    pub fn debug_flags(&self) -> DebugFeatureFlags {
        self.debug_flags
    }

    /// Initialize and start a simulation level with the given configuration
    ///
    /// Sets up all level-specific resources including RNG seed, diner providers,
    /// spawning parameters, and static world objects. Must be called before
    /// the first tick() to properly initialize the simulation state.
    pub fn start(&mut self, level: LevelConfig) {
        const DEFAULT_TIMESTEP_S: f64 = 0.1;
        self.world.insert_resource(Time::new(DEFAULT_TIMESTEP_S));
        self.world.insert_resource(GameRng::new(level.seed));

        let db = (*self.world.resource::<GameModelRegistryRes>()).clone();
        let canteen = db.canteens.get_by_id(&level.canteen).unwrap();

        let world_size = canteen.size();
        self.world
            .insert_resource(NavigationGrid::new(world_size, 0.1).into_res());
        self.world
            .insert_resource(CrowdCostField::new(world_size, 0.1).into_res());

        self.world.insert_resource(Canteen {
            model: canteen.clone(),
        });
        self.world.insert_resource(DinerProvider {
            model: level.diner_provider.clone(),
        });
        let spawner_model = level.diner_spawner.clone();
        let mut curve = spawner_model.spawn_curve.clone();
        curve.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal));
        curve.dedup_by(|a, b| (a.time - b.time).abs() <= f32::EPSILON);
        if curve.is_empty() {
            curve.push(SpawnRateKey {
                time: 0.0,
                multiplier: 1.0,
            });
        } else if curve[0].time > 0.0 {
            let initial_multiplier = curve[0].multiplier;
            curve.insert(
                0,
                SpawnRateKey {
                    time: 0.0,
                    multiplier: initial_multiplier,
                },
            );
        }

        self.world.insert_resource(DinerSpawner {
            model: spawner_model,
            curve,
            next_spawn_timer: 0.0,
            next_diner_id: 0,
            spawning_finished: false,
        });
        self.world.insert_resource(DayStatus::default());
        self.world.insert_resource(level.into_res());

        // Spawn static objects once at startup
        let mut schedule = Schedule::default();
        schedule.add_systems((spawn_static_objects, build_collision_grid));
        schedule.run(&mut self.world);
    }

    /// Advance the simulation by one time step
    ///
    /// Executes all registered systems in the proper order to update
    /// entity states, handle interactions, and progress the simulation.
    /// This should be called at regular intervals to maintain simulation flow.
    pub fn tick(&mut self) {
        // Check if time resource exists and advance it, or create a new one
        let mut time = self.world.resource_mut::<Time>();
        time.tick();

        self.schedule.run(&mut self.world);
    }

    /// Retrieve all events that occurred during the last simulation step
    pub fn poll_events(&mut self) -> Vec<Event> {
        vec![] // Placeholder
    }

    /// Create a snapshot of the current simulation state for serialization or debugging
    pub fn snapshot(&mut self) -> Snapshot {
        let mut query = self
            .world
            .query::<(Entity, &DisplayState, &mut Transform)>();
        let display = query
            .iter_mut(&mut self.world)
            .map(|(e, d, mut t)| DisplaySnapshot {
                core_id: EntityId(NonZero::new(e.to_bits()).unwrap()),
                proto: d.proto.clone(),
                transform: t.snapshot(),
            })
            .collect();

        let movement_debug = self.snapshot_movement();
        let collision_debug = self.snapshot_collision();
        let crowd_debug = self.snapshot_crowd();

        Snapshot {
            display,
            movement_debug,
            collision_debug,
            crowd_debug,
        }
    }

    /// Get the root entity of the display hierarchy
    pub fn root_entity(&self) -> EntityId {
        EntityId(NonZero::new(self.world.resource::<DisplayRoot>().0.to_bits()).unwrap())
    }

    /// Check if the current day is complete (spawning finished and all diners left)
    pub fn is_day_complete(&self) -> bool {
        let (day_status, spawner) = (
            self.world.resource::<DayStatus>(),
            self.world.resource::<DinerSpawner>(),
        );
        day_status.current_diner_count == 0 && spawner.spawning_finished
    }
}
