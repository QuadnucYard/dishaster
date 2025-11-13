//! Opening animation simulation

use dishaster_opening_models::OpeningConfig;
use dishrupt_ecs::display::DisplayRoot;
use dishrupt_simulation::{ISimulation, SimulationFeature};

use crate::{prelude::*, protocol::*, resources::OpeningAssetsRes};

/// Opening animation simulation engine
pub struct Simulation {
    /// ECS world
    world: World,
    /// System schedule
    schedule: Schedule,
}

impl Simulation {
    /// Create a new opening simulation
    pub fn new(config: OpeningConfig, seed: u64) -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        let root_entity = world.spawn(Transform::default()).id();
        world.insert_resource(DisplayRoot(root_entity));

        // Add configuration resources
        world.insert_resource(config.world.into_res());
        world.insert_resource(config.assets.into_res());

        world.insert_resource(SpawnTimers::default());
        world.insert_resource(WorldRng::new(seed));
        world.insert_resource(DeltaTime::default());
        world.insert_resource(EventQueue::default());

        // Add systems
        schedule.add_systems(
            (
                crate::systems::spawn_foods,
                crate::systems::spawn_faces,
                crate::systems::spawn_texts,
                crate::systems::update_physics,
                crate::systems::update_texts,
                crate::systems::despawn_out_of_bounds,
            )
                .chain(),
        );

        Self { world, schedule }
    }

    /// Update the simulation
    pub fn update(&mut self, delta: f32) {
        // Update time resource
        if let Some(mut time) = self.world.get_resource_mut::<DeltaTime>() {
            time.delta = delta;
        }

        // Run systems
        self.schedule.run(&mut self.world);
    }

    /// Get reference to the ECS world
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get mutable reference to the ECS world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Generate display snapshots for Stage to present
    pub fn snapshot(&mut self) -> Snapshot {
        let world = self.world_mut();
        let mut display_snapshots = Vec::new();
        let mut presentation_snapshots = Vec::new();

        // Query all entities with Position component
        let mut query = world.query::<(
            Entity,
            &Position,
            Option<&FoodObject>,
            Option<&FaceObject>,
            Option<&TextObject>,
            Option<&Rotation>,
            Option<&Scale>,
            Option<&ColorTint>,
            Option<&Alpha>,
            Option<&WavePhase>,
        )>();

        for (entity, pos, food, face, text, rotation, scale, color, alpha, wave) in
            query.iter(world)
        {
            let core_id = entity.to_entity_id();

            // Determine prefab based on entity type. Prefer component proto when present.
            let assets = world.resource::<OpeningAssetsRes>();
            let (proto, item_type) = if food.is_some() {
                (assets.food_prefab.clone(), ItemType::Food)
            } else if face.is_some() {
                (assets.face_prefab.clone(), ItemType::Face)
            } else if text.is_some() {
                // Use configured text prefab
                (assets.text_prefab.clone(), ItemType::Text)
            } else {
                continue;
            };

            let position = pos.0.extend(0.0);
            let scale_val = scale.map(|s| s.0).unwrap_or(1.0);
            let rotation_val = rotation.map(|r| r.0).unwrap_or(0.0);

            let display_snapshot = DisplaySnapshot {
                core_id,
                proto,
                name: None,
                transform: TransformSnapshot {
                    position,
                    scale: Vec3::splat(scale_val),
                    rotation: rotation_val,
                    parent: None,
                },
            };

            display_snapshots.push(display_snapshot);

            // Create presentation snapshot for dynamic visual updates
            let presentation_snapshot = ObjectSnapshot {
                entity: core_id,
                item_type,
                alpha: alpha.map(|a| a.0).unwrap_or(1.0),
                wave_phase: wave.map(|w| w.0).unwrap_or(0.0),
                color: color.map(|c| (c.r, c.g, c.b)),
            };

            presentation_snapshots.push(presentation_snapshot);
        }

        Snapshot {
            display: display_snapshots,
            objects: presentation_snapshots,
        }
    }
}

/// Opening simulation feature definition
pub struct OpeningSimulationFeat;

impl SimulationFeature for OpeningSimulationFeat {
    type Snapshot = Snapshot;
    type Command = ();
    type Query = ();
    type Event = SimEvent;
    type Response = ();
    type Profile = ();
}

impl ISimulation<OpeningSimulationFeat> for Simulation {
    fn root_entity(&self) -> EntityId {
        self.world.resource::<DisplayRoot>().0.to_entity_id()
    }

    fn tick(&mut self) {
        self.update(1.0 / 60.0);
    }

    fn snapshot(&mut self) -> Snapshot {
        self.snapshot()
    }

    fn poll_events(&mut self) -> Vec<SimEvent> {
        self.world.resource_mut::<EventQueue>().drain()
    }

    fn poll_responses(&mut self) -> Vec<()> {
        Vec::new()
    }

    fn command(&mut self, _command: ()) {
        // No commands to process
    }

    fn query(&mut self, _query: ()) {
        // No queries to process
    }

    fn persist(&mut self) {}
}
