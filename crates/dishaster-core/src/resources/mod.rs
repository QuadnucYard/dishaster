//! Simulation resources and global state management

mod buffers;
mod time;

use std::{collections::VecDeque, sync::Arc};

use dishaster_save_models::PermanentEffects;

pub use self::{buffers::*, time::Time};
use crate::{components::*, models::*, prelude::*};

#[allow(missing_docs)]
pub struct NavigationRngTag;
/// SystemRng specialized for navigation systems
pub type NavigationRng = SystemRng<NavigationRngTag>;

#[allow(missing_docs)]
pub struct QueueingRngTag;
/// SystemRng specialized for queueing systems
pub type QueueingRng = SystemRng<QueueingRngTag>;

#[allow(missing_docs)]
pub struct ServingRngTag;
/// SystemRng specialized for serving systems
pub type ServingRng = SystemRng<ServingRngTag>;

#[allow(missing_docs)]
pub struct CrabRngTag;
/// SystemRng specialized for crab trial systems
pub type CrabRng = SystemRng<CrabRngTag>;

/// Global canteen configuration and layout information
///
/// Contains the physical layout, dimensions, entrance/exit locations,
/// and window positions for the dining hall. Acts as the spatial
/// foundation for all simulation activities.
#[derive(Resource)]
pub struct Canteen {
    /// Static configuration model defining canteen layout and properties
    pub model: CanteenModel,
}

/// Day progression and completion tracking system
///
/// Monitors the state of the current simulation day, tracking active
/// diners and determining when daily objectives are met. Coordinates
/// with spawning systems to determine overall day completion.
#[derive(Resource, Default)]
pub struct DayStatus {
    /// Seed for the day's RNG
    pub seed: Seed,
    /// Current day index
    pub current_day: Day,
    /// Starting day index
    pub start_day: Day,
    /// Simulation time when service begins (seconds since midnight)
    pub start_time: Seconds,

    /// Whether the current day has reached completion criteria
    pub completed: bool,
    /// Whether the day completion event has been emitted
    pub completion_emitted: bool,
    /// Number of diners currently active in the canteen
    pub live_diner_count: u32,
}

/// Statistics collected during a single day
#[derive(Resource, Default)]
pub struct DailyStats {
    /// Total number of diner visits since the start of the day
    pub total_visits: u32,
    /// Total food consumed in kilograms
    pub total_consumption_kg: f32,
    /// Total revenue collected in currency units
    pub total_revenue: f32,
    /// Number of diners who completed their meal
    pub completed_diners: u32,
}

/// Resource wrapper for Arc<GameModelRegistry>
pub type GameModelRegistryRes = ResWrapper<Arc<GameModelRegistry>>;

/// Permanent effects from management decisions stored in player profile
pub type PermanentEffectsRes = ResWrapper<PermanentEffects>;

/// Daily scheduling state (generated each day from PersistentDinerPool)
///
/// Manages today's scheduled arrivals and active diners.
#[derive(Resource, Default)]
pub struct DailyDinerSchedule {
    /// List of diners scheduled to arrive today with their arrival times
    scheduled_diners: VecDeque<ScheduledDiner>,
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
    /// Current psychological state affecting decision-making
    pub psych_state: PsychState,
    /// The diner's accumulated memory of past dining experiences
    pub long_term_memory: LongTermMemory,
    /// Visual appearance (cosmetics)
    pub appearance: Appearance,

    // Session-specific attributes (generated per visit)
    /// The simulation time when this diner should be spawned (seconds from day start)
    pub arrival_time: Seconds,
    /// Budget allocated for this meal (calculated once at spawn with randomness)
    pub meal_budget: f32,
}

impl DailyDinerSchedule {
    pub fn new(mut scheduled: Vec<ScheduledDiner>) -> Self {
        // Sort by arrival time
        scheduled.sort_by(|a, b| {
            a.arrival_time
                .partial_cmp(&b.arrival_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Self {
            scheduled_diners: scheduled.into(),
        }
    }

    /// Check if there are more diners to spawn
    pub fn has_pending_spawns(&self) -> bool {
        !self.scheduled_diners.is_empty()
    }

    /// Mark all remaining diners as spawned
    pub fn finish_spawning(&mut self) {
        self.scheduled_diners.clear();
    }

    /// Get the next diner to spawn if arrival time has passed
    pub fn next_diner_if_ready(&mut self, current_time: f32) -> Option<ScheduledDiner> {
        self.scheduled_diners
            .pop_front_if(|next| current_time >= next.arrival_time)
    }

    /// Add multiple scheduled diners to the schedule
    pub fn add_many(&mut self, diners: Vec<ScheduledDiner>) {
        for diner in diners {
            self.scheduled_diners.push_back(diner);
        }
        self.scheduled_diners.make_contiguous().sort_by(|a, b| {
            a.arrival_time
                .partial_cmp(&b.arrival_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

pub type TrialSession = ResWrapper<dishaster_trial::TrialSession>;

/// Crab turmoil trial tracking for the current day
#[derive(Resource)]
pub struct CrabTurmoil {
    /// Probability of triggering a crab trial for eligible diners today
    pub probability: f32,
    /// Maximum number of diners who can trigger crab trials today
    pub trigger_limit: u32,
    /// Set of diners who have already triggered crab trials today
    pub triggered_diners: FxHashSet<Entity>,
}

/// Pending inspector visit incident to be applied at some time of the day
#[derive(Resource)]
pub struct PendingInspectorVisit {
    /// Inspector visit model
    pub model: InspectorVisitModel,
    /// Scheduled time for the inspector visit (seconds from run start)
    pub scheduled_time: Seconds,
}

#[derive(Resource)]
pub struct ManagementDecisions {
    pub available: Vec<ManagementDecisionModel>,
}

/// Reputation and food safety state tracking
///
/// Tracks canteen reputation, food safety risk index, and food quality.
/// Updated daily based on feedback and management decisions.
pub type ReputationStateRes = ResWrapper<ReputationState>;

/// Reputation system configuration
pub type ReputationConfigRes = ResWrapper<ReputationConfig>;

/// Ordering system configuration
pub type OrderingConfigRes = ResWrapper<OrderingConfig>;

/// Decision system configuration
pub type DecisionConfigRes = ResWrapper<DecisionConfig>;
