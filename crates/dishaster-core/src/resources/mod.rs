//! Simulation resources and global state management

mod buffers;
mod time;

use std::{collections::VecDeque, sync::Arc};

pub use buffers::*;
pub use time::Time;

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
    /// Number of diners currently active in the canteen
    pub live_diner_count: usize,
    /// Total number of diner visits since the start of the day
    pub total_visits: usize,
    /// Whether the spawning period has begun for the current day
    pub started: bool,
    /// Whether the current day has reached completion criteria
    pub completed: bool,
    /// Whether the day completion event has been emitted
    pub completion_emitted: bool,
}

/// Resource wrapper for Arc<GameModelRegistry>
pub type GameModelRegistryRes = ResWrapper<Arc<GameModelRegistry>>;

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
            temperature: 0.8,
        }
    }

    /// Reset the session for a new trial
    pub fn reset(&mut self) {
        self.asked_questions.clear();
        self.last_response_index = None;
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
    }
}

#[derive(Resource)]
pub struct ManagementDecisions {
    pub available: Vec<ManagementDecisionModel>,
}
