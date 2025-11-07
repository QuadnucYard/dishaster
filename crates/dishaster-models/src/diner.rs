use super::prelude::*;

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

/// Current psychological state affecting decision-making
///
/// This represents runtime state that changes during a dining session.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PsychState {
    /// Current hunger level (0..1, higher = more hungry)
    pub hunger: f32,
    /// Current mood (-1..1, negative = bad mood, positive = good mood)
    pub mood: f32,
    /// Current patience threshold in seconds (dynamically adjusted)
    pub patience: f32,
    /// Trust in the canteen (0..1, affects tolerance to issues)
    pub trust: f32,
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

/// Short-term memory for current meal session
#[derive(Debug, Clone, Default)]
pub struct ShortTermMemory {
    /// Windows that have been observed this session
    pub seen_windows: FxHashSet<ModelId>,
    /// Dishes tried in current meal
    pub tried_dishes: Vec<ModelId>,
    /// Perceived price references updated by seeing prices
    pub expected_prices: FxHashMap<ModelId, f32>,
}
