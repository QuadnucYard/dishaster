use crate::{CanteenLayoutState, Day, DinerPool, PermanentEffects, Seed, prelude::*};

/// Current version of the progress schema stored on disk.
pub const USER_PROGRESS_VERSION: u32 = 1;

/// Persistent representation of the player's long-term progress.
///
/// The structure only contains stable data that must survive across sessions.
/// Transient entities live solely inside the simulation and never appear here.
#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    /// Metadata describing file format and timestamps.
    pub meta: ProfileMeta,

    /// Player-specific counters and unlock tracking.
    pub progress: Option<PlayerProgress>,

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
    pub permanent_effects: PermanentEffects,

    /// Set of hint IDs that have been shown to the player (persisted).
    #[serde(default)]
    pub seen_hints: FxHashSet<EcoString>,

    /// Set of endings that have been unlocked by the player.
    #[serde(default)]
    pub achieved_endings: FxHashSet<EcoString>,
}

/// Metadata stored alongside the progress payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    /// Format schema version to support migrations.
    pub version: u32,
    /// Creation timestamp in UTC seconds.
    pub created_at_utc: u64,
    /// Last updated timestamp in UTC seconds.
    pub updated_at_utc: u64,
}

/// Player-centric state that drives level selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgress {
    /// The level played.
    pub level_id: ModelId,

    /// Day index.
    pub current_day: Day,

    /// Reputation score used for balancing future systems.
    pub reputation: f32,

    /// Base seed for deterministic day generation.
    pub rng_seed: Seed,
}

/// Lifetime statistics collected for dashboards and analytics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregateStats {
    /// Total number of diner visits.
    pub lifetime_visits: u32,
    /// Total number of diners served.
    pub lifetime_served: u32,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayStats {
    /// Day index when these stats were recorded.
    pub day: Day,
    /// Total number of diner visits.
    pub total_visits: u32,
    /// Number of diners who completed their meal.
    pub completed_diners: u32,
    /// Total revenue collected.
    pub revenue: f32,
    /// Total food consumed in kilograms.
    pub consumption_kg: f32,
}

impl AggregateStats {
    /// Update aggregate stats with data from a completed day.
    pub fn update(&mut self, day_stats: &DayStats) {
        self.lifetime_visits += day_stats.total_visits;
        self.lifetime_served += day_stats.completed_diners;
        self.lifetime_revenue += day_stats.revenue as f64;
        self.lifetime_consumption_kg += day_stats.consumption_kg as f64;
    }
}

/// Type of game ending reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndingType {
    /// Bad ending: Reputation dropped to 0 (forced).
    BadReputation,
    /// Good ending: Reputation reached 100 (optional).
    GoodReputation,
    /// Bad ending: Food safety shutdown (forced).
    Rectification,
}

impl EndingType {
    /// Get string identifier for this ending type (for localization).
    pub fn id(self) -> &'static str {
        match self {
            EndingType::BadReputation => "bad_reputation",
            EndingType::GoodReputation => "good_reputation",
            EndingType::Rectification => "rectification",
        }
    }
}
