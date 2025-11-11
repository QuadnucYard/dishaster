use dishaster_models::*;

use crate::{components::Movement, prelude::*};

#[allow(missing_docs)]
#[derive(Bundle)]
pub struct DinerBundle {
    pub diner: Diner,

    pub state: DinerState,
    pub goal: DinerGoalState,
    pub targets: DinerTargets,

    pub personality: DinerPersonality,
    pub dining_profile: DinerDiningProfile,
    pub psych_state: DinerPsychState,
    pub ltm: DinerLongTermMemory,
    pub stm: DinerShortTermMemory,
    pub appearance: DinerAppearance,

    pub movement: Movement,
}

/// Core diner identity
#[derive(Component)]
pub struct Diner {
    /// Unique identifier for this diner instance. Useless yet.
    #[allow(dead_code)]
    pub id: u32,
}

/// Current state of the diner
#[derive(Component, Default)]
pub struct DinerState {
    /// Entity of the tray held by the diner, if any
    pub tray: Option<Entity>,
    /// Entity of the chopsticks held by the diner, if any
    pub chopsticks: Option<Entity>,
    /// Served dishes held by the diner (can hold multiple)
    pub served_dishes: Vec<ServedDish>,
    /// Total amount spent on current meal (accumulated)
    pub total_spent: f32,
    /// Budget allocated for this meal
    #[allow(dead_code)] // Will be used for budget constraints in future
    pub meal_budget: f32,
}

/// What gets served to a diner
#[derive(Debug)]
pub struct ServedDish {
    /// The entity of the served dish
    pub entity: Entity,
    /// Original dish reference
    pub dish_id: ModelId,
    /// Actual weight of the portion served (kg)
    #[allow(dead_code)] // Will be used for nutrition tracking in future
    pub served_weight: f32,
    /// Actual values at time of service
    #[allow(unused)]
    pub served_quantity: f32,
    /// Quality level when served
    pub served_quality: f32,
    /// Final price charged to customer (calculated from weight if ByWeight)
    pub price_paid: f32,
    /// Time taken to serve this dish
    #[allow(unused)]
    pub service_time: Seconds,
    /// Any contamination
    pub contamination_level: f32,
}

/// Current goal state of the diner
#[derive(Component)]
pub struct DinerGoalState {
    /// Current goal
    current: DinerGoal,
    /// Optional next goal to transition to in the next update
    pending: Option<DinerGoal>,
    /// State entry time for timing. Internal use only.
    pub timer: f32,
}

impl Default for DinerGoalState {
    fn default() -> Self {
        Self {
            current: DinerGoal::Enter,
            pending: None,
            timer: 0.0,
        }
    }
}

impl DinerGoalState {
    /// Check if the diner is currently in the given goal state.
    pub fn is(&self, goal: DinerGoal) -> bool {
        self.current == goal
    }

    /// Get the current goal state.
    pub fn current(&self) -> DinerGoal {
        self.current
    }

    /// Transition to the next goal state, resetting the timer.
    pub fn update(&mut self, next_goal: DinerGoal) {
        self.pending = Some(next_goal);
    }

    /// Step the internal timer and apply any pending goal transitions.
    pub fn step(&mut self, delta_time: f32) {
        if let Some(next) = self.pending.take() {
            self.current = next;
            self.timer = 0.0;
        }
        self.timer += delta_time;
    }

    /// Reset the state timer to zero.
    pub fn reset_timer(&mut self) {
        self.timer = 0.0;
    }
}

/// Possible goals for a diner in the canteen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinerGoal {
    /// Just spawned, moving into the canteen
    Enter,
    /// Observing available windows to decide
    Observe,
    /// Deciding which window to approach
    DecideWindow,
    /// Moving to pick up a tray
    PickTray,
    /// Moving to pick up chopsticks
    PickChopsticks,
    /// Moving to the chosen window
    QueueForWindow,
    /// Being served at the window
    GetServed,
    /// Moving to find a seat
    FindSeat,
    /// Moving to the assigned seat
    MoveToSeat,
    /// Eating the meal at the seat
    Eat,
    /// Moving to the dish return area
    ReturnDishes,
    /// Leaving the canteen
    Leave,
}

/// Diner's current targets and decisions
#[derive(Component, Default)]
pub struct DinerTargets {
    /// Window the diner is currently observing
    pub observing_window: Option<Entity>,
    /// Currently chosen window entity
    pub chosen_window: Option<Entity>,
    /// Currently chosen tray dispenser entity
    pub tray_target: Option<Entity>,
    /// Currently chosen chopstick dispenser entity
    pub chopstick_target: Option<Entity>,
    /// Currently chosen table entity and seat index
    pub chosen_seat: Option<(Entity, usize)>,
    /// Dish collector the diner will visit after eating
    pub collector_target: Option<Entity>,
    /// Exit target position to leave the canteen
    pub exit_target: Option<()>,
}

/// Wrapper component for Personality
pub type DinerPersonality = CompWrapper<Personality>;

/// Wrapper component for DiningProfile
pub type DinerDiningProfile = CompWrapper<DiningProfile>;

/// Wrapper component for PsychState
pub type DinerPsychState = CompWrapper<PsychState>;

/// Wrapper component for LongTermMemory
pub type DinerLongTermMemory = CompWrapper<LongTermMemory>;

/// Short-term memory for current meal (not serialized, session only)
pub type DinerShortTermMemory = CompWrapper<ShortTermMemory>;

/// Wrapper component for Appearance (cosmetic customization)
pub type DinerAppearance = CompWrapper<Appearance>;

/// Current psychological state affecting decision-making
///
/// This represents runtime state that changes during a dining session.
#[derive(Debug, Clone)]
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

/// Short-term memory for current meal session
#[derive(Debug, Clone, Default)]
pub struct ShortTermMemory {
    /// Windows that have been observed this session
    #[allow(dead_code)] // For future window exploration behavior
    pub seen_windows: FxHashSet<ModelId>,
    /// Dishes tried in current meal
    #[allow(dead_code)] // For future variety tracking
    pub tried_dishes: Vec<ModelId>,
    /// Perceived price references updated by seeing prices
    #[allow(dead_code)] // For future price learning
    pub expected_prices: FxHashMap<ModelId, f32>,
    /// Initial dining intentions formed after entering
    /// Maps dish_id to desire weight (higher = want more)
    pub dish_intentions: FxHashMap<ModelId, f32>,
    /// Dishes currently ordered in this serving session
    pub current_order: Vec<ModelId>,
}
