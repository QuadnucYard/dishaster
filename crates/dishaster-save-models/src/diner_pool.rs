//! Common data structures for diner pool management
//!
//! These types are shared between simulation (core) and persistence layers
//! to coordinate diner scheduling and memory tracking across days.

use crate::{Appearance, prelude::*};

/// Persistent pool of all diner profiles across days
///
/// This pool only grows (or is pruned externally). It contains all known diners.
/// Each day, core decides which diners visit based on their memories.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DinerPool {
    /// All known diner profiles (grows over time)
    pub profiles: Vec<DinerProfile>,
}

/// Runtime representation of a diner profile in the persistent pool
///
/// This type is used by the simulation core to track diners across days.
/// It mirrors the persistence layer's DinerProfile but is not directly serialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DinerProfile {
    /// Unique identifier for this diner (persisted across days)
    pub id: u32,
    /// Day of last visit (0-indexed)
    pub last_visit_day: u32,
    /// Total number of visits
    pub total_visits: u32,
    /// Personality traits controlling behavior (may evolve slightly over time)
    pub personality: Personality,
    /// Dining-specific behavioral profile
    pub dining_profile: DiningProfile,
    /// Long-term memory of dining experiences
    pub long_term_memory: LongTermMemory,
    /// Visual appearance (cosmetics)
    pub appearance: Appearance,
}

/// Fixed personality traits that shape diner behavior
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Personality {
    /// Price sensitivity (0..1, higher = more price sensitive)
    pub frugality: f32,
    /// Willingness to try new dishes (0..1, higher = more adventurous)
    pub adventurous: f32,
    /// Likelihood to question or complain about issues (0..1)
    pub confrontational: f32,
    /// Base patience in seconds
    pub patience_base: f32,
    /// Decision-making speed (0..1, higher = faster decisions)
    pub decisiveness: f32,
    /// Adaptability to changes (0..1, higher = more flexible)
    pub adaptiveness: f32,
}

/// Dining-specific behavioral profile
///
/// These parameters affect dining experience but are separate from core personality.
/// They represent learned or physiological characteristics related to eating.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiningProfile {
    /// Economic capacity (how much they can/will spend)
    pub economic_capacity: f32,
    /// Eating speed multiplier (0.5 = slow, 1.0 = normal, 1.5 = fast)
    /// Actual eating time = dish_base_time / eating_speed
    pub eating_speed: f32,
    /// Preferred arrival time range (seconds from day start)
    pub preferred_arrival_time: (f32, f32),
}

/// Long-term memory persisted across multiple visits
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LongTermMemory {
    /// Preference weights for dish tags (-1..1 per tag)
    /// Positive values indicate preference, negative indicate dislike
    pub like_tags: FxHashMap<EcoString, f32>,
    /// Experience with specific dishes
    pub dish_experience: FxHashMap<ModelId, DishMemory>,
    /// Overall satisfaction with this canteen (0..1)
    pub overall_like: f32,
}

/// Memory of a specific dish from previous experiences
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DishMemory {
    /// Number of times this dish was eaten
    pub times_eaten: u32,
    /// Average rating/satisfaction from eating this dish (-1..1)
    pub avg_rating: f32,
    /// Last time this dish was eaten (simulation time)
    pub last_eaten: Option<f32>,
}
