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

    /// Whether the spawning period has begun for the current day
    pub started: bool,
    /// Whether the current day has reached completion criteria
    pub completed: bool,
    /// Whether the day completion event has been emitted
    pub completion_emitted: bool,
    /// Number of diners currently active in the canteen
    pub live_diner_count: usize,
}

/// Statistics collected during a single day
#[derive(Resource, Default)]
pub struct DailyStats {
    /// Total number of diner visits since the start of the day
    pub total_visits: usize,
    /// Total food consumed in kilograms
    pub total_consumption_kg: f32,
    /// Total revenue collected in currency units
    pub total_revenue: f32,
    /// Number of diners who completed their meal
    pub completed_diners: usize,
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
}

/// Trial session state tracking to avoid repetition and improve coherence
#[derive(Resource)]
pub struct TrialSession {
    /// Pseudorandom number generator for trial session
    pub rng: Prng,
    /// Indices of questions already asked in this trial
    asked_questions: Vec<usize>,
    /// The most recent response corpus index selected by the player
    pub last_response_index: Option<usize>,
    /// The most recent diner speech index
    pub last_diner_speech_index: Option<usize>,
    /// The most recent diner speech index that the player is responding to (for context evaluation)
    pub current_question_index: Option<usize>,
    /// Current continuation depth (consecutive speeches by same speaker)
    pub continuation_depth: u32,
    /// Maximum allowed continuation depth before forcing speaker alternation
    pub max_continuation_depth: u32,
    /// Temperature parameter for sampling (higher = more random, lower = more deterministic)
    pub temperature: f32,
}

impl TrialSession {
    /// Create a new trial session with default temperature
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Prng::new(seed),
            asked_questions: Vec::new(),
            last_response_index: None,
            last_diner_speech_index: None,
            current_question_index: None,
            continuation_depth: 0,
            max_continuation_depth: 3,
            temperature: 0.8,
        }
    }

    /// Reset the session for a new trial
    pub fn reset(&mut self) {
        self.asked_questions.clear();
        self.last_response_index = None;
        self.last_diner_speech_index = None;
        self.current_question_index = None;
        self.continuation_depth = 0;
    }

    /// Check if a question has been asked
    pub fn has_asked(&self, question_index: usize) -> bool {
        self.asked_questions.contains(&question_index)
    }

    /// Mark a question as asked
    pub fn mark_asked(&mut self, question_index: usize) {
        if !self.has_asked(question_index) {
            self.asked_questions.push(question_index);
        }
    }

    /// Record the player's response choice
    pub fn set_last_response(&mut self, response_index: usize) {
        self.last_response_index = Some(response_index);
        // Reset continuation depth when speaker alternates
        self.continuation_depth = 0;
    }

    /// Record a diner speech
    pub fn set_last_diner_speech(&mut self, speech_index: usize) {
        self.last_diner_speech_index = Some(speech_index);
    }

    /// Set the current question index (the question player is responding to)
    pub fn set_current_question(&mut self, question_index: usize) {
        self.current_question_index = Some(question_index);
    }

    /// Increment continuation depth
    #[allow(unused)]
    pub fn increment_continuation(&mut self) {
        self.continuation_depth += 1;
    }

    /// Check if continuation is allowed (not at max depth)
    pub fn can_continue(&self) -> bool {
        self.continuation_depth < self.max_continuation_depth
    }

    /// Reset continuation depth (when alternating speakers)
    pub fn reset_continuation(&mut self) {
        self.continuation_depth = 0;
    }

    /// Decide whether to continue based on best continuation score
    /// Uses the score as probability (higher score = more likely to continue)
    pub fn should_continue(&mut self, best_score: f32) -> bool {
        if !self.can_continue() {
            return false;
        }

        // Use score directly as probability (already normalized 0-1 from embedding similarity)
        // Low scores (<0.3) rarely continue, high scores (>0.7) usually continue
        let prob = best_score.clamp(0.0, 1.0);
        self.rng.random::<f32>() < prob
    }
}

#[derive(Resource)]
pub struct ManagementDecisions {
    pub available: Vec<ManagementDecisionModel>,
}

/// Parameters for ordering decisions
#[derive(Debug, Clone, Resource)]
pub struct OrderingConfig {
    /// Tolerance for "close enough" to desired satiation (fraction of diner's max)
    /// When sat_needed <= tolerance × max_satiation, stop ordering
    pub satiation_tolerance: f32,
    /// Maximum number of different dishes one person can order
    pub max_dishes_per_order: usize,
    /// Weight for taste/preference in scoring (0..1)
    pub taste_weight: f32,
    /// Weight for quality in scoring (0..1)
    pub quality_weight: f32,
    /// Variety penalty factor (penalizes repeated dishes)
    pub variety_beta: f32,
    /// Sigmoid steepness for taste score
    pub sigmoid_k: f32,
    /// Maximum budget overspend factor (e.g., 1.2 = can spend up to 120% of budget)
    pub max_budget_overspend: f32,
    /// Base probability of accepting over-budget dish (0..1)
    pub overspend_base_prob: f32,
}

impl Default for OrderingConfig {
    fn default() -> Self {
        Self {
            satiation_tolerance: 0.05, // Stop when within 5% of desired satiation
            max_dishes_per_order: 4,
            taste_weight: 0.6,
            quality_weight: 0.4,
            variety_beta: 0.6,
            sigmoid_k: 2.0,
            max_budget_overspend: 1.3, // Can exceed budget by up to 30%
            overspend_base_prob: 0.3,  // 30% base chance to accept overspend
        }
    }
}

/// Reputation and food safety state tracking
///
/// Tracks canteen reputation, food safety risk index, and food quality.
/// Updated daily based on feedback and management decisions.
pub type ReputationStateRes = ResWrapper<ReputationState>;

/// Reputation system configuration
pub type ReputationConfigRes = ResWrapper<ReputationConfig>;
