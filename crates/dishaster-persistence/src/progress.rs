use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use dishaster_models::DinerPool;
use dishrupt_persistence::Persistable;
use serde::{Deserialize, Serialize};

/// Current version of the progress schema stored on disk.
pub const USER_PROGRESS_VERSION: u32 = 1;

/// Persistent representation of the player's long-term progress.
///
/// The structure only contains stable data that must survive across sessions.
/// Transient entities live solely inside the simulation and never appear here.
#[derive(Serialize, Deserialize)]
pub struct UserProgress {
    /// Metadata describing file format and timestamps.
    pub meta: ProgressMeta,
    /// Player-specific counters and unlock tracking.
    pub player: PlayerProgress,
    /// Customized canteen layout modifications.
    pub canteen_layout: CanteenLayoutState,
    /// Cumulative statistics for analytics and balancing.
    pub stats_aggregate: AggregateStats,
    /// Aggregated memory about diners to drive future generation.
    #[serde(default)]
    pub diner_pool: DinerPool,
}

/// Metadata stored alongside the progress payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressMeta {
    /// Format schema version to support migrations.
    pub version: u32,
    /// Creation timestamp in UTC seconds.
    pub created_at_utc: u64,
    /// Last updated timestamp in UTC seconds.
    pub updated_at_utc: u64,
}

/// Player-centric state that drives level selection.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerProgress {
    /// Day index starting from one.
    pub current_day: u32,
    /// Reputation score used for balancing future systems.
    pub reputation: f32,
    /// Base seed for deterministic day generation.
    pub rng_seed: u64,
}

/// Snapshot of user-authored canteen layout changes.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CanteenLayoutState {}

/// Lifetime statistics collected for dashboards and analytics.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AggregateStats {
    /// Accumulated profit across all completed days.
    pub lifetime_profit: f64,
    /// Total number of diners served.
    pub lifetime_served: u64,
    /// Total safety incidents recorded.
    pub safety_incidents: u32,
    /// Average satisfaction of the most recent day.
    pub last_day_avg_satisfaction: f32,
}

impl UserProgress {
    /// Create a brand-new progress record for first-time players.
    pub fn new(seed: u64) -> Self {
        let now = now_unix();
        Self {
            meta: ProgressMeta {
                version: USER_PROGRESS_VERSION,
                created_at_utc: now,
                updated_at_utc: now,
            },
            player: PlayerProgress {
                current_day: 1,
                reputation: 50.0,
                rng_seed: seed,
            },
            canteen_layout: Default::default(),
            diner_pool: Default::default(),
            stats_aggregate: Default::default(),
        }
    }
}

pub(crate) fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Persistable for UserProgress {
    fn from_bytes(data: Vec<u8>) -> Result<Self>
    where
        Self: Sized,
    {
        ron::de::from_bytes(&data).context("parse user progress RON")
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let ron_str = ron::ser::to_string_pretty(self, Default::default())?;
        Ok(ron_str.into_bytes())
    }
}
