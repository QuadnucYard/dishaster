//! Opening animation simulation

use dishrupt_ecs::display::DisplayRoot;
use dishrupt_simulation::{ISimulation, SimulationFeature};

use crate::{prelude::*, resources::OpeningAssets};

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

        world.insert_resource(config);
        // Insert configurable assets (dish/emoji prefabs and review texts)
        world.insert_resource(OpeningAssets::default());
        world.insert_resource(SpawnTimers::default());
        world.insert_resource(WorldRng::new(seed));
        world.insert_resource(DeltaTime::default());

        // Add systems
        schedule.add_systems(
            (
                crate::systems::spawn_dishes,
                crate::systems::spawn_emojis,
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
        let mut snapshots = Vec::new();

        // Query all entities with Position component
        let mut query = world.query::<(
            Entity,
            &Position,
            Option<&DishIcon>,
            Option<&EmojiIcon>,
            Option<&ReviewText>,
            Option<&Rotation>,
            Option<&Scale>,
        )>();

        for (entity, pos, dish, emoji, text, rotation, scale) in query.iter(world) {
            let core_id = entity.to_entity_id();

            // Determine prefab based on entity type. Prefer component proto when present.
            let assets = world.resource::<OpeningAssets>();
            let proto = if let Some(d) = dish {
                d.proto.clone()
            } else if let Some(e) = emoji {
                e.proto.clone()
            } else if text.is_some() {
                // Use configured text prefab
                assets.text_prefab.clone()
            } else {
                continue;
            };

            let position = pos.0.extend(0.0);
            let scale_val = scale.map(|s| s.0).unwrap_or(1.0);
            let rotation_val = rotation.map(|r| r.0).unwrap_or(0.0);

            let name = text.map(|t| EcoString::from(t.content.as_str().to_string()));

            let snapshot = DisplaySnapshot {
                core_id,
                proto,
                name,
                transform: TransformSnapshot {
                    position,
                    scale: Vec3::splat(scale_val),
                    rotation: rotation_val,
                    parent: None,
                },
            };

            snapshots.push(snapshot);
        }

        Snapshot { display: snapshots }
    }
}

pub struct Snapshot {
    pub display: Vec<DisplaySnapshot>,
}

/// Opening simulation feature definition
pub struct OpeningSimulationFeat;

impl SimulationFeature for OpeningSimulationFeat {
    type Snapshot = Snapshot;
    type Command = ();
    type Query = ();
    type Event = ();
    type Response = ();
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

    fn poll_events(&mut self) -> Vec<()> {
        Vec::new()
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
}
