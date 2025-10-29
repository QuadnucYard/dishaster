//! Common data structures for diner pool management
//!
//! These types are shared between simulation (core) and persistence layers
//! to coordinate diner scheduling and memory tracking across days.

use super::{DiningProfile, LongTermMemory, Personality, prelude::*};

/// Persistent pool of all diner profiles across days
///
/// This pool only grows (or is pruned externally). It contains all known diners.
/// Each day, core decides which diners visit based on their memories.
#[derive(Default, Serialize, Deserialize)]
pub struct DinerPool {
    /// Configuration for memory decay and spawning behavior
    pub config: DinerPoolConfig,
    /// All known diner profiles (grows over time)
    pub profiles: Vec<DinerProfile>,
}

/// Configuration for pool behavior and decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DinerPoolConfig {
    /// Initial pool size when creating from scratch
    pub initial_pool_size: usize,
    /// Memory decay rate per day (0..1, e.g., 0.95 = 5% decay)
    pub memory_decay_rate: f32,
    /// Tag preference decay rate per day (0..1, e.g., 0.98 = 2% decay)
    pub tag_decay_rate: f32,
    /// Probability modifier for satisfied diners (overall_like >= 0.0)
    pub high_satisfaction_visit_rate: f32,
    /// Probability modifier for dissatisfied diners (overall_like < 0.0)
    pub low_satisfaction_visit_rate: f32,
}

impl Default for DinerPoolConfig {
    fn default() -> Self {
        Self {
            initial_pool_size: 1000,
            memory_decay_rate: 0.95,
            tag_decay_rate: 0.98,
            high_satisfaction_visit_rate: 0.75,
            low_satisfaction_visit_rate: 0.25,
        }
    }
}

/// Runtime representation of a diner profile in the persistent pool
///
/// This type is used by the simulation core to track diners across days.
/// It mirrors the persistence layer's DinerProfile but is not directly serialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DinerProfile {
    /// Unique identifier for this diner (persisted across days)
    pub id: u32,
    /// Personality traits controlling behavior (may evolve slightly over time)
    pub personality: Personality,
    /// Dining-specific behavioral profile
    pub dining_profile: DiningProfile,
    /// Long-term memory of dining experiences
    pub long_term_memory: LongTermMemory,
    /// Day of last visit (0-indexed)
    pub last_visit_day: u32,
    /// Total number of visits
    pub total_visits: u32,
}

/// A diner scheduled to arrive at a specific simulation time
///
/// Created during day initialization from persisted profiles or generated
/// as new customers. Contains all information needed to spawn a diner entity
/// at the designated time.
#[derive(Debug, Clone)]
pub struct ScheduledDiner {
    /// Profile ID
    pub id: u32,
    /// The personality traits controlling decision-making behavior
    pub personality: Personality,
    /// Dining-specific behavioral profile
    pub dining_profile: DiningProfile,
    /// The diner's accumulated memory of past dining experiences
    pub long_term_memory: LongTermMemory,

    // Session-specific attributes (generated per visit)
    /// The simulation time when this diner should be spawned (seconds from day start)
    pub arrival_time: Seconds,
    /// Initial hunger level for this visit (randomized)
    pub hunger: f32,
}
