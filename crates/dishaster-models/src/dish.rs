use dishaster_save_models::PricingMethod;
use dishrupt_core::display::DisplayModel;

use super::prelude::*;

// ===================== Core Dish Models =====================

/// Static dish definition - the master "recipe" for a food item
#[derive(Debug, Clone, Deserialize)]
pub struct DishModel {
    /// Unique identifier for this dish type
    pub id: ModelId,
    /// Base characteristics - can be extended without breaking changes
    pub characteristics: DishCharacteristics,
    /// Prefab used to render this dish in debug or game views.
    pub display: DisplayModel,
}

impl HasId for DishModel {
    fn id(&self) -> &ModelId {
        &self.id
    }
}

/// Core characteristics that define how a dish behaves in the simulation
#[derive(Debug, Clone, Deserialize)]
pub struct DishCharacteristics {
    /// Quality range this dish can achieve
    pub quality_range: MinMax<f32>,
    /// Base risk level (affects contamination)
    #[serde(default)]
    pub risk_level: f32,
    /// Base serving time
    pub serving_time: Seconds,
    /// Tags for categorization and preference matching (e.g., "meat", "spicy", "soup")
    #[serde(default)]
    pub tags: Vec<EcoString>,
    /// Reference price for value comparison (used in diner decision-making)
    #[serde(default)]
    pub base_price: f32,

    /// Weight distribution for portion sizes
    #[serde(default)]
    pub weight_distrib: DishWeightDistribution,
    /// Satiation contribution per kilogram (how filling this dish is per kg)
    #[serde(default = "default_satiation_per_kg")]
    pub satiation_per_kg: f32,
    /// Typical eating time per kilogram (seconds/kg)
    ///
    /// This represents how long it takes to eat 1kg of this dish type.
    /// The actual eating time is: weight × eating_time_per_kg / eating_speed
    ///
    /// Realistic values:
    /// - Rice/mantou: 120-150 s/kg (solid starches take time to chew)
    /// - Stir-fried dishes: 180-240 s/kg (need to pick up and chew)
    /// - Soups: 300-400 s/kg (sipping takes longer despite liquid form)
    /// - Noodles: 150-200 s/kg (easier to eat than rice but still solid)
    /// - Dumplings: 200-250 s/kg (need to bite, chew carefully)
    #[serde(default = "default_eating_time_per_kg")]
    pub eating_time_per_kg: f32,
}

/// Weight distribution parameters for dish portions
#[derive(Debug, Clone, Deserialize)]
pub struct DishWeightDistribution {
    /// Mean weight of a standard portion (kg)
    pub mean: f32,
    /// Standard deviation of portion weight (kg)
    pub stddev: f32,
}

impl Default for DishWeightDistribution {
    fn default() -> Self {
        DishWeightDistribution {
            mean: 0.15,   // 150g standard portion (typical 大伙 dish: 100-200g)
            stddev: 0.03, // ±30g variation
        }
    }
}

/// Default satiation per kilogram
///
/// Calibrated for student diners who typically order 2 dishes (sometimes 2-4):
/// - 150g dish (0.15kg) → 22.5 satiation units (~23% of max 100)
/// - 200g dish (0.2kg) → 30 units (30% of max)
/// - Typical 2-dish meal (300g) → 45 satiation (~45% of max)
/// - Large 3-dish meal (450g) → 67.5 satiation (~68% of max)
/// - Maximum 4-dish meal (600g) → 90 satiation (90% of max)
///
/// This means students feel satisfied with 2 dishes, very full with 3-4 dishes.
fn default_satiation_per_kg() -> f32 {
    150.0 // 150 satiation units per kg
}

/// Default eating time per kilogram (seconds/kg)
///
/// Calibrated for typical Chinese canteen meals where diners finish in 10-20 minutes.
/// Realistic calibration:
/// - Typical meal: 350g (2 dishes + rice)
/// - Average eating time: 15 minutes = 900 seconds
/// - Base rate: 900s / 0.35kg ≈ 2571 s/kg
/// - With eating_speed variation (0.5-1.5):
///   - Slow eater (0.5×): ~30 min for 350g meal
///   - Normal (1.0×): ~15 min for 350g meal
///   - Fast eater (1.5×): ~10 min for 350g meal
fn default_eating_time_per_kg() -> f32 {
    2500.0 // ~2500 seconds per kg (realistic canteen pace)
}

// ===================== Window Service Models =====================

/// Categories of food service windows with different capabilities
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum WindowType {
    /// Standard food service window
    General,
    /// Specialized dishes requiring special equipment
    Specialty,
    /// Liquid-based dishes like soups
    Soup,
}

/// Static window service template - defines what this window type can do
#[derive(Debug, Clone, Deserialize)]
pub struct WindowServiceModel {
    /// Unique identifier for this service template
    pub id: ModelId,
    /// Type category of this window
    pub window_type: WindowType,
    /// Physical layout
    pub layout: WindowLayout,
    /// Dishes offered at this service window, which will be randomly selected from
    /// to populate the actual menu during gameplay
    pub dish_options: Vec<PricedDish>,
    /// Display model
    pub display: DisplayModel,
}

impl HasId for WindowServiceModel {
    fn id(&self) -> &ModelId {
        &self.id
    }
}

/// Physical layout configuration for a service window
#[derive(Debug, Clone, Deserialize)]
pub struct WindowLayout {
    /// Physical size of the service window
    pub size: Size,
    /// X-axis positions for customer queueing
    pub queue_x: Vec<Meters>,
    /// Positions where dishes can be placed. Relative to the top-left corner.
    pub dish_slots: Vec<Rect>,
}

/// Dish offered at a service window with a specific price
#[derive(Debug, Clone, Deserialize)]
pub struct PricedDish {
    /// Reference to the dish model
    pub dish_id: ModelId,
    /// Price charged for this dish at the service window
    pub pricing: PricingMethod,
}
