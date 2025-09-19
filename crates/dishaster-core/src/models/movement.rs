use super::prelude::*;

/// Static configuration model for Movement
#[derive(Component, Debug, Clone, Deserialize, Serialize)]
pub struct MovementModel {
    /// Movement speed in units per second
    pub movement_speed: f32,
    /// Avoidance speed when avoiding obstacles
    pub avoidance_speed: f32,
    /// Distance threshold for considering target reached
    pub arrival_threshold: f32,
}
