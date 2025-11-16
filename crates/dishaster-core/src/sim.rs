//! Main simulation engine and event handling

use std::sync::Arc;

use dishaster_interface::{snapshots::*, *};
use dishaster_navigation::*;
use dishaster_save_models::LevelSetupState;
// Re-export simulation trait
pub use dishrupt_simulation::ISimulation;

use crate::{
    components::*,
    messages::*,
    models::*,
    prelude::*,
    resources::*,
    systems::{self, *},
};

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
    pub(crate) debug_flags: DebugFlags,
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

        systems::register_lifecycle_systems(&mut world);
        schedule.add_systems(
            (
                // Recompute soft crowd costs for pathfinding
                update_crowd_field,
                // Update movement speeds dynamically based on crowd, urgency, carry weight
                update_movement_speeds,
                // Spawn logic may add diners
                update_diner_spawner,
                // Deliver delayed serving communications before state updates
                process_serving_messages,
                // Handle dish served events
                on_dish_served,
                // Agents decide targets and compute paths
                dining_systems(),
                // Allocate staff and schedule service events
                drive_serving_sessions,
                detect_dispenser_stock_change,
                handle_refill_request,
                handle_refill_staff,
                // Update queue ordering and slot targets before movement
                (update_queue_members, update_queue_intents),
                // Move agents along paths
                run_path_requests,
                update_agent_movement,
                // Sync visuals to movement positions
                sync_transform_with_movement,
                check_day_completion,
                // Presentation related systems
                feedback_present_system,
                monitor_reputation_changes.run_if(resource_changed::<ReputationStateRes>),
            )
                .chain(),
        );

        Self {
            world,
            schedule,
            debug_flags: DebugFlags::none(),
        }
    }

    /// Update the debug feature configuration for snapshot export.
    pub fn set_debug_flags(&mut self, flags: DebugFlags) {
        self.debug_flags = flags;
    }

    /// Retrieve the current debug feature configuration.
    pub fn debug_flags(&self) -> DebugFlags {
        self.debug_flags
    }

    /// Initialize and start a simulation level with the given configuration
    ///
    /// Sets up all level-specific resources including RNG seed, diner providers,
    /// spawning parameters, and static world objects. Must be called before
    /// the first tick() to properly initialize the simulation state.
    pub fn start(&mut self, level: LevelSetupState) {
        const DEFAULT_TIMESTEP_S: f64 = 0.1;

        self.world.insert_resource(Time::new(DEFAULT_TIMESTEP_S));

        let mut world_rng = WorldRng::new(level.seed.get());

        let db = Arc::clone(self.world.resource::<GameModelRegistryRes>());
        let level_config = db
            .levels
            .get_by_id(&level.level_id)
            .expect("Invalid level ID");
        let canteen = db
            .canteens
            .get_by_id(&level_config.canteen)
            .expect("Invalid canteen ID");

        let world_size = canteen.size();
        self.world
            .insert_resource(NavigationGrid::new(world_size, 0.1).into_res());

        self.world.insert_resource(Canteen {
            model: canteen.clone(),
        });

        self.world.insert_resource(EventQueue::default());
        self.world.insert_resource(ResponseQueue::default());
        self.world.insert_resource(ServingCommsQueue::default());

        self.world
            .insert_resource(db.ordering_config.clone().into_res());
        self.world
            .insert_resource(db.decision_config.clone().into_res());
        self.world
            .insert_resource(db.reputation_config.clone().into_res());

        self.world
            .insert_resource(ReputationState::default().into_res());
        self.world.insert_resource(DayStatus {
            seed: level.seed,
            current_day: level.day,
            start_day: level_config.start_day,
            ..Default::default()
        });
        self.world.insert_resource(DailyStats::default());
        self.world.insert_resource(
            dishaster_trial::TrialSession::new(world_rng.derive_seed()).into_res(),
        );
        self.world
            .insert_resource(level.permanent_effects.clone().into_res());
        self.world.insert_resource(level.into_res());
        // Derived RNGs
        self.world
            .insert_resource(NavigationRng::new(world_rng.derive_seed()));
        self.world
            .insert_resource(QueueingRng::new(world_rng.derive_seed()));
        self.world
            .insert_resource(ServingRng::new(world_rng.derive_seed()));
        self.world.insert_resource(world_rng);

        self.startup();
    }

    /// Perform initial setup tasks at simulation startup
    fn startup(&mut self) {
        // Add messages
        self.world
            .add_message::<DishServed>()
            .add_message::<FeedbackMessage>()
            .add_message::<RefillDispenser>();

        // Add observers for agent spawn/despawn events to log presentation events
        self.world.add_observer(
            |event: On<Add, AgentTag>,
             query: Query<&DinerAppearance>,
             mut events: ResMut<EventQueue>| {
                let appearance = query.get(event.entity).ok().map(|a| a.to_view().into());

                events.push(SimEvent::AgentSpawned {
                    entity: event.entity.to_entity_id(),
                    appearance,
                });
            },
        );
        self.world.add_observer(
            |_event: On<Add, Diner>, mut daily_stats: ResMut<DailyStats>| {
                daily_stats.total_visits += 1; // Increment total visits on diner spawn
            },
        );

        self.world.add_observer(
            |event: On<Remove, AgentTag>, mut events: ResMut<EventQueue>| {
                events.push(SimEvent::AgentDespawned(event.entity.to_entity_id()));
            },
        );

        // Spawn static objects once at startup
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                on_day_started,
                initial_spawning_systems(),
                build_collision_grid,
                populate_diner_pool,
                set_daily_schedule,
            )
                .chain(),
        );
        schedule.run(&mut self.world);
    }
}

impl ISimulation<CoreSimulationFeat> for Simulation {
    /// Advance the simulation by one time step
    ///
    /// Executes all registered systems in the proper order to update
    /// entity states, handle interactions, and progress the simulation.
    /// This should be called at regular intervals to maintain simulation flow.
    fn tick(&mut self) {
        let mut time = self.world.resource_mut::<Time>();
        time.tick();

        self.schedule.run(&mut self.world);
    }

    /// Create a snapshot of the current simulation state for serialization or debugging
    ///
    /// This function is expected to be idempotent: multiple calls between ticks
    /// should yield identical results.
    fn snapshot(&mut self) -> Snapshot {
        let stats = self.snapshot_stats();
        let display = self.snapshot_display();
        let debug = self.snapshot_debug();

        Snapshot {
            stats,
            display,
            debug,
        }
    }

    fn poll_events(&mut self) -> Vec<SimEvent> {
        self.world.resource_mut::<EventQueue>().drain()
    }

    fn poll_responses(&mut self) -> Vec<SimResponse> {
        self.world.resource_mut::<ResponseQueue>().drain()
    }

    fn root_entity(&self) -> EntityId {
        self.world.resource::<DisplayRoot>().0.to_entity_id()
    }

    fn command(&mut self, command: SimCommand) {
        self.handle_command(command);
    }

    fn query(&mut self, command: SimQuery) {
        self.handle_query(command);
    }

    fn persist(&mut self) -> SimProfile {
        let mut persist_system = IntoSystem::into_system(systems::persist_system);
        persist_system.initialize(&mut self.world);
        persist_system
            .run((), &mut self.world)
            .expect("failed to persist simulation")
    }
}

impl Simulation {
    /// Check if the current day is complete (spawning finished and all diners left)
    pub fn is_day_complete(&self) -> bool {
        let (day_status, schedule) = (
            self.world.resource::<DayStatus>(),
            self.world.resource::<DailyDinerSchedule>(),
        );
        day_status.live_diner_count == 0 && !schedule.has_pending_spawns()
    }
}
