//! Simulation commands and control interfaces.

/// Commands that can be sent to the simulation from external sources.
pub enum SimCommand {
    /// Start a new run (spawning diners, etc.)
    StartRun,
    /// Finish the current run immediately.
    EndRun,
}
