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
}

/// Different pricing strategies for dishes
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum PricingMethod {
    /// Fixed price per serving
    PerPortion(f32),
    /// Price calculated by weight (per kg)
    ByWeight(f32),
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
    /// Display name for this service window
    pub name: EcoString,
    /// Dishes this window type can serve
    pub compatible_dishes: Vec<ModelId>,
    /// Physical layout
    pub layout: WindowLayout,
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

// ===================== Operational Configuration =====================

/// Player's configuration for a specific window instance
#[derive(Debug, Clone, Deserialize)]
pub struct WindowConfiguration {
    /// Which slot to use
    pub slot_index: usize,
    /// Which service template this uses
    pub service_template: ModelId,
    /// Whether enabled
    pub is_enabled: bool,
    /// Player-selected dishes
    pub dish_assignments: Vec<DishAssignment>,
}

/// Player's assignment of a dish to a specific slot in a window
#[derive(Debug, Clone, Deserialize)]
pub struct DishAssignment {
    /// Which slot to use
    pub slot_index: usize,
    /// Which dish to serve
    pub dish_id: ModelId,
    /// Player-set pricing
    pub pricing: PricingConfig,
}

/// Player-configured pricing for a dish assignment
#[derive(Debug, Clone, Deserialize)]
pub struct PricingConfig {
    /// Base price set by player
    pub method: PricingMethod,
}

/// Pricing adjustments applied to base dish prices
#[derive(Debug, Clone, Deserialize)]
pub struct PriceModifier {
    /// Type of modifier ("discount", "markup", "time_based", etc.)
    pub modifier_type: EcoString, // "discount", "markup", "time_based", etc.
    /// Modifier value or percentage
    pub value: f32,
}

// ===================== Runtime State =====================

/// Runtime state of an active dish in a window
#[derive(Debug, Clone)]
pub struct ActiveDish {
    /// Configuration this came from
    pub assignment: DishAssignment,
    /// Current runtime state
    pub state: DishRuntimeState,
}

/// Dynamic state tracking for dishes during simulation
#[derive(Debug, Clone)]
pub struct DishRuntimeState {
    /// Current quantity available
    pub current_quantity: f32,
    /// Current quality (may degrade)
    pub current_quality: f32,
    /// Current contamination level
    pub contamination_level: f32,
    /// When last restocked
    pub last_restocked: f32,
    /// Times served today
    pub service_count: u32,
}

/// What gets served to a diner - minimal and focused
#[derive(Debug, Clone, Deserialize)]
pub struct ServedDish {
    /// Original dish reference
    pub dish_id: ModelId,
    /// Name for display
    pub name: EcoString,
    /// Actual values at time of service
    pub served_quantity: f32,
    /// Quality level when served
    pub served_quality: f32,
    /// Final price charged to customer
    pub price_paid: f32,
    /// Time taken to serve this dish
    pub service_time: Seconds,
    /// Any contamination
    pub contamination_level: f32,
}
