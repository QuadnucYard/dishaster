//! Resources for opening animation simulation

use crate::{
    models::{OpeningAssets, OpeningWorldConfig},
    prelude::*,
    protocol::SimEvent,
};

/// Configuration for opening animation
pub type OpeningConfigRes = ResWrapper<OpeningWorldConfig>;

/// Assets and configurable content for opening animation
pub type OpeningAssetsRes = ResWrapper<OpeningAssets>;

/// Spawn timers resource
#[derive(Resource, Debug, Default)]
pub struct SpawnTimers {
    /// Timer for food spawning
    pub food: f32,
    /// Timer for face spawning
    pub face: f32,
    /// Timer for text spawning
    pub text: f32,
}

/// Simple time resource for delta-time updates
#[derive(Resource, Debug, Default)]
pub struct DeltaTime {
    /// Delta time in seconds
    pub delta: f32,
}

/// Event queue for simulation events
pub type EventQueue = MessageQueue<SimEvent>;
