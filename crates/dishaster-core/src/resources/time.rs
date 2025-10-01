use crate::prelude::*;

/// High-precision time management system for the simulation
///
/// Uses tick-based timing to avoid floating-point accumulation errors.
/// Time is calculated from discrete ticks rather than accumulated deltas,
/// ensuring consistent precision throughout long simulations.
///
/// NOTE: the time is simulation time in the simulation world, not real clock time.
#[derive(Resource)]
pub struct Time {
    /// Current tick number in the current simulation frame
    pub current_tick: u32,
    /// Total accumulated ticks since simulation start (persistent across resets)
    pub total_ticks: u32,
    /// Duration of one simulation tick in seconds (configurable precision)
    pub tick_duration: f64,
    /// Current simulation time calculated from ticks (tick_count * tick_duration)
    pub current_time: f64,
}

impl Time {
    /// Create a new time system with specified tick duration
    pub fn new(tick_duration: f64) -> Self {
        Self {
            current_tick: 0,
            total_ticks: 0,
            tick_duration,
            current_time: 0.0,
        }
    }

    /// Advance time by one tick
    ///
    /// Updates current_tick, total_ticks, and recalculates current_time
    /// from tick count to maintain precision and avoid accumulation errors.
    pub fn tick(&mut self) {
        self.current_tick += 1;
        self.total_ticks += 1;
        // Calculate current time from tick count to avoid floating point accumulation errors
        self.current_time = self.current_tick as f64 * self.tick_duration;
    }

    /// Get current simulation time in seconds as high-precision f64
    pub fn current_time_seconds(&self) -> f64 {
        self.current_time
    }

    /// Get current simulation time as f32 for systems requiring float precision
    pub fn current_time_f32(&self) -> f32 {
        self.current_time as f32
    }

    /// Reset time state to initial values
    ///
    /// Resets current_tick, total_ticks, and current_time to zero
    /// while preserving the original tick_duration.
    pub fn reset(&mut self) {
        self.current_tick = 0;
        self.total_ticks = 0;
        self.current_time = 0.0;
    }
}
