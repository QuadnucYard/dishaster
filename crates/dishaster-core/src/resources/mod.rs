//! Simulation resources and global state management

mod time;

use std::{collections::HashMap, sync::Arc};

use rand_chacha::ChaCha8Rng;
pub use time::Time;

use crate::{models::*, prelude::*, snapshots::*};

/// Turn a type into a Bevy resource
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ResWrapper<T>(T);

impl<T> From<T> for ResWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Extension trait to convert any type into a ResourceWrapper
pub trait IntoResource {
    /// Wrap this value in a ResourceWrapper for use as a Bevy resource
    fn into_res(self) -> ResWrapper<Self>
    where
        Self: Sized,
    {
        ResWrapper::from(self)
    }
}

impl<T> IntoResource for T {}

/// Root entity for all display-related objects in the scene
#[derive(Resource)]
pub struct DisplayRoot(pub Entity);

/// Cryptographically secure random number generator for deterministic simulation
///
/// Uses ChaCha8 algorithm to ensure reproducible randomness across simulation runs
/// when initialized with the same seed. Essential for deterministic gameplay
/// and testing scenarios.
#[derive(Resource, Deref, DerefMut)]
pub struct GameRng(ChaCha8Rng);

impl GameRng {
    /// Create a new deterministic RNG from a 64-bit seed
    pub fn new(seed: u64) -> Self {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[..8].copy_from_slice(&seed.to_le_bytes());
        Self(ChaCha8Rng::from_seed(seed_bytes))
    }
}

/// Global canteen configuration and layout information
///
/// Contains the physical layout, dimensions, entrance/exit locations,
/// and window positions for the dining hall. Acts as the spatial
/// foundation for all simulation activities.
#[derive(Resource)]
pub struct Canteen {
    /// Static configuration model defining canteen layout and properties
    pub model: CanteenModel,
}

/// Diner generation template and behavioral parameter provider
///
/// Defines the statistical ranges and probability distributions used
/// to generate diverse diner personalities, attributes, and behaviors.
/// Each generated diner derives their characteristics from these templates.
#[derive(Resource)]
pub struct DinerProvider {
    /// Configuration model containing generation parameters and ranges
    pub model: DinerProviderModel,
}

/// Diner spawning controller managing timing and flow control
///
/// Handles when and how frequently new diners enter the canteen.
/// Tracks spawn timing, completion status, and coordinates with
/// the day completion logic.
#[derive(Resource)]
pub struct DinerSpawner {
    /// Configuration model defining spawn timing and flow parameters
    pub model: DinerSpawnerModel,
    /// Sorted spawn rate curve for quick lookup
    pub curve: Vec<SpawnRateKey>,
    /// Countdown timer until next diner spawn (in seconds)
    pub next_spawn_timer: f64,
    /// Unique ID for the next diner to be spawned
    pub next_diner_id: u32,
    /// Whether the spawning period has ended for the current day
    pub spawning_finished: bool,
}

impl DinerSpawner {
    /// Check if the spawning period has completed based on simulation time
    pub fn is_spawning_complete(&self, current_time: f64) -> bool {
        current_time >= self.model.run_length as f64
    }

    /// Sample the next inter-arrival interval (seconds) using a time-varying Poisson rate.
    pub fn sample_next_interval(&self, rng: &mut GameRng, current_time: f64) -> f64 {
        let lambda = self.arrival_rate_per_sec(current_time).max(1.0e-6);
        let u = rng.random::<f64>().clamp(f64::EPSILON, 1.0 - f64::EPSILON);
        -u.ln() / lambda
    }

    fn arrival_rate_per_sec(&self, time: f64) -> f64 {
        let base = (self.model.base_rate_per_min.max(0.0) as f64) / 60.0;
        base * self.rate_multiplier(time)
    }

    fn rate_multiplier(&self, time: f64) -> f64 {
        if self.curve.is_empty() {
            return 1.0;
        }

        let mut prev = &self.curve[0];
        if time <= prev.time as f64 {
            return prev.multiplier.max(0.0) as f64;
        }

        for point in self.curve.iter().skip(1) {
            if time <= point.time as f64 {
                let duration = (point.time - prev.time).max(f32::EPSILON);
                let t = ((time as f32 - prev.time).max(0.0) / duration).clamp(0.0, 1.0);
                let start = prev.multiplier.max(0.0) as f64;
                let end = point.multiplier.max(0.0) as f64;
                return start + (end - start) * f64::from(t);
            }
            prev = point;
        }

        self.curve
            .last()
            .map(|point| point.multiplier.max(0.0) as f64)
            .unwrap_or(1.0)
    }
}

/// Day progression and completion tracking system
///
/// Monitors the state of the current simulation day, tracking active
/// diners and determining when daily objectives are met. Coordinates
/// with spawning systems to determine overall day completion.
#[derive(Resource, Default)]
pub struct DayStatus {
    /// Number of diners currently active in the canteen
    pub current_diner_count: usize,
}

/// Resource wrapper for Arc<GameModelRegistry>
pub type GameModelRegistryRes = ResWrapper<Arc<GameModelRegistry>>;

/// Resource wrapper for LevelConfig
pub type LevelConfigRes = ResWrapper<LevelConfig>;

/// Log of presentation events to be processed by the display system
#[derive(Resource, Default)]
pub struct EventLog(pub(crate) Vec<PresentationEvent>);

impl EventLog {
    /// Record a new presentation event
    pub fn emit(&mut self, event: PresentationEvent) {
        self.0.push(event);
    }

    /// Retrieve and clear all logged events
    pub fn drain(&mut self) -> Vec<PresentationEvent> {
        std::mem::take(&mut self.0)
    }
}

/// Bi-directional lookup between service windows and their assigned serving staff.
#[derive(Resource, Default)]
pub struct ServingStaffRegistry {
    window_to_staff: HashMap<Entity, Vec<Entity>>,
}

impl ServingStaffRegistry {
    /// Register a staff entity responsible for a given window.
    pub fn register(&mut self, window: Entity, staff: Entity) {
        self.window_to_staff.entry(window).or_default().push(staff);
    }

    /// Resolve every staff entity assigned to the provided window.
    pub fn staff_for(&self, window: Entity) -> Option<&[Entity]> {
        self.window_to_staff.get(&window).map(Vec::as_slice)
    }

    /// Remove the staff mapping when the window shuts down or the staff despawns.
    pub fn unregister(&mut self, window: Entity) {
        self.window_to_staff.remove(&window);
    }
}
