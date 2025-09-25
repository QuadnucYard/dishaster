//! Simulation-wide constants (all lengths are in meters)
//! Keep numbers centralized to avoid magic numbers scattered in logic.

use crate::models::Meters;

/// Physical width/height of a diner collider (square) in meters
pub const DINER_COLLIDER_SIZE: Meters = 0.4;

/// How close counts as "arrived" when observing a spot (meters)
pub const OBSERVATION_ARRIVAL_EPS: Meters = 1.5;

/// How close counts as "arrived" when queuing at a window (meters)
pub const QUEUE_ARRIVAL_EPS: Meters = 0.6;

/// Radius used when picking a wander/observation target (meters)
pub const WANDER_RADIUS: Meters = 6.0;

/// Attempts when searching for a valid (non-colliding) random spot
pub const FIND_SPOT_ATTEMPTS: usize = 12;

/// Forward offset (in +Y) from the window line to approach and order (meters)
/// Represents the distance a diner stands from the counter to order.
pub const WINDOW_APPROACH_OFFSET: Meters = 0.6;

/// How close to an exit counts as leaving (meters)
pub const EXIT_ARRIVAL_EPS: Meters = 1.0;

/// Typical human walking speed in meters per second (Chinese canteen context)
pub const DINER_SPEED_MPS: Meters = 1.35;

/// Waypoint arrival tolerance when following a path (meters)
pub const PATH_WAYPOINT_EPS: Meters = 0.3;

/// Default per-diner initial satisfaction [0,1]
pub const DEFAULT_DINER_SATISFACTION: f32 = 0.5;

/// Default runtime dish values
pub const DEFAULT_DISH_QUANTITY: f32 = 100.0;
/// Quality in [0,1], where 1 is perfect and 0 is inedible
pub const DEFAULT_DISH_QUALITY: f32 = 0.8;
/// Contamination level in [0,1], where 0 is clean and 1 is hazardous
pub const DEFAULT_DISH_CONTAMINATION: f32 = 0.0;
/// Seconds since epoch or day start; 0 means "not set yet" for our sim
pub const DEFAULT_DISH_LAST_RESTOCKED_S: f32 = 0.0;
