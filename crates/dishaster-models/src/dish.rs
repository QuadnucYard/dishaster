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
