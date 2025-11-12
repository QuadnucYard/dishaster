use crate::{CanteenLayoutState, Day, DinerPool, Seed, prelude::*};

/// Current version of the progress schema stored on disk.
pub const USER_PROGRESS_VERSION: u32 = 1;

/// Persistent representation of the player's long-term progress.
///
/// The structure only contains stable data that must survive across sessions.
/// Transient entities live solely inside the simulation and never appear here.
#[derive(Serialize, Deserialize)]
pub struct PlayerProfile {
    /// Metadata describing file format and timestamps.
    pub meta: ProfileMeta,

    /// Player-specific counters and unlock tracking.
    pub progress: PlayerProgress,

    /// Cumulative statistics for analytics and balancing.
    #[serde(default)]
    pub aggregates: AggregateStats,

    /// Day-by-day statistics history.
    #[serde(default)]
    pub daily_history: Vec<DayStats>,

    /// Customized canteen layout modifications.
    pub layout: CanteenLayoutState,

    /// Aggregated memory about diners to drive future generation.
    #[serde(default)]
    pub diner_pool: DinerPool,

    /// Permanent effects from management decisions.
    #[serde(default)]
    pub permanent_effects: crate::PermanentEffects,
}

/// Metadata stored alongside the progress payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileMeta {
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
    /// The level played.
    pub level_id: ModelId,

    /// Day index.
    pub current_day: Day,

    /// Reputation score used for balancing future systems.
    pub reputation: f32,

    /// Base seed for deterministic day generation.
    pub rng_seed: Seed,

    /// Set of hint IDs that have been shown to the player (persisted).
    #[serde(default)]
    pub seen_hints: FxHashSet<EcoString>,
}

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
    /// Total revenue collected across all days.
    pub lifetime_revenue: f64,
    /// Total food consumed across all days in kilograms.
    pub lifetime_consumption_kg: f64,
}

/// Per-day statistics history for tracking performance over time.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DayStats {
    /// Day index when these stats were recorded.
    pub day: Day,
    /// Total number of diner visits.
    pub total_visits: usize,
    /// Number of diners who completed their meal.
    pub completed_diners: usize,
    /// Total revenue collected.
    pub revenue: f32,
    /// Total food consumed in kilograms.
    pub consumption_kg: f32,
}
