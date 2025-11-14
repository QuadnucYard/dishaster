//! Scene transition effects for smooth visual scene changes.
//!
//! Provides a generic transition system that can be extended with different effects.
//! The default implementation provides fade-to-black transitions.

mod fade;

pub use self::fade::FadeTransition;

/// Trait for scene transition effects.
///
/// Implement this trait to create custom transition effects (fade, slide, wipe, etc.).
/// The scene stack uses this trait to coordinate scene changes with visual effects.
///
/// # Lifecycle
///
/// 1. Scene change requested → `transition_out()` called
/// 2. Effect plays (e.g., fade to black)
/// 3. Effect completes → `is_transitioning()` returns false
/// 4. Scene stack processes the change (load new scene)
/// 5. `transition_in()` called
/// 6. Effect reverses (e.g., fade from black)
pub trait SceneTransition {
    /// Start the transition-out effect (before scene change).
    ///
    /// # Parameters
    /// - `duration`: Optional effect duration in seconds
    ///
    /// # Returns
    /// The actual duration used for the effect.
    fn transition_out(&mut self, duration: Option<f32>) -> f32;

    /// Start the transition-in effect (after scene change).
    ///
    /// # Parameters
    /// - `duration`: Optional effect duration in seconds
    ///
    /// # Returns
    /// The actual duration used for the effect.
    fn transition_in(&mut self, duration: Option<f32>) -> f32;

    /// Check if a transition is currently in progress.
    ///
    /// Returns `false` when the effect has completed.
    fn is_transitioning(&self) -> bool;

    /// Process transition state (must be called every frame).
    ///
    /// Updates animation state and detects when transitions complete.
    fn process(&mut self);
}
