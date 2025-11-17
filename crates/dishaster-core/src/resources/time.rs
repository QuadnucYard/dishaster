use crate::prelude::*;

/// High-precision time management system for the simulation
///
/// Uses tick-based timing to avoid floating-point accumulation errors.
/// Time is calculated from discrete ticks rather than accumulated deltas,
/// ensuring consistent precision throughout long simulations.
#[derive(Resource)]
pub struct Time {
    /// Current tick number in the current simulation frame. It should always advance by 1 each tick.
    pub current_tick: Tick,
    /// Duration of one simulation tick in seconds (configurable precision)
    pub tick_duration: f64,
    /// Elapsed simulation time, in tune with current_tick
    pub current_time: f64,
    /// Time offset for world time calculations
    entry_time: f64,
    /// Current world clock time in seconds since midnight
    pub world_time: f64,
}

impl Time {
    /// Create a new time system with specified tick duration and entry time
    pub fn new(tick_duration: f64, entry_time: f64) -> Self {
        Self {
            current_tick: 0,
            tick_duration,
            current_time: 0.0,
            entry_time,
            world_time: entry_time,
        }
    }

    /// Advance time by one tick
    ///
    /// Updates current_tick and recalculates current_time from tick count
    /// to maintain precision and avoid accumulation errors.
    pub fn tick(&mut self) {
        self.current_tick += 1;
        // Calculate current time from tick count to avoid floating point accumulation errors
        self.current_time = self.current_tick as f64 * self.tick_duration;
        // World time is entry_time + elapsed simulation time
        self.world_time = self.entry_time + self.current_time;
    }

    /// Fast-forward time to the specified target if current time is earlier
    pub fn fast_forward_to(&mut self, target_time: f64) {
        if self.world_time < target_time {
            self.world_time = target_time;
            self.entry_time = target_time - self.current_time;
        }
    }
}
