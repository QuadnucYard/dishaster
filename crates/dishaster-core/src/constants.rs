//! Simulation-wide constants (all lengths are in meters)
//! Keep numbers centralized to avoid magic numbers scattered in logic.

use crate::models::Meters;

/// How close counts as "arrived" when observing a spot (meters)
pub const OBSERVATION_ARRIVAL_EPS: Meters = 1.5;

/// How close counts as "arrived" when queuing at a window (meters)
pub const QUEUE_ARRIVAL_EPS: Meters = 0.6;

/// Spacing between customers standing in the same queue (meters)
pub const QUEUE_SPACING: Meters = 0.5;

/// Radius used when picking a wander/observation target (meters)
pub const WANDER_RADIUS: Meters = 6.0;

/// Attempts when searching for a valid (non-colliding) random spot
pub const FIND_SPOT_ATTEMPTS: usize = 12;

/// Forward offset (in +Y) from the window line to approach and order (meters)
/// Represents the distance a diner stands from the counter to order.
pub const WINDOW_APPROACH_OFFSET: Meters = 0.6;

/// How close to an exit counts as leaving (meters)
pub const EXIT_ARRIVAL_EPS: Meters = 1.0;

/// Waypoint arrival tolerance when following a path (meters)
pub const PATH_WAYPOINT_EPS: Meters = 0.3;

/// Default per-diner initial satisfaction [0,1]
pub const DEFAULT_DINER_SATISFACTION: f32 = 0.5;

/// Placeholder service duration at the window (seconds)
pub const PLACEHOLDER_SERVICE_TIME_S: f32 = 10.0;

/// Soft margin from canteen walls for free roaming movement (meters)
pub const DINING_AREA_MARGIN: Meters = 1.0;

/// How close counts as sitting down at a table (meters)
pub const TABLE_SEAT_ARRIVAL_EPS: Meters = 0.45;

/// Time a diner spends eating before leaving the table (seconds)
pub const BASE_EATING_DURATION_S: f32 = 90.0;

/// Dirtiness increase applied to a table after a diner finishes eating
pub const TABLE_DIRTINESS_INCREMENT: f32 = 0.2;

/// Maximum allowed dirtiness value
pub const TABLE_MAX_DIRTINESS: f32 = 1.0;

/// Tolerance when reaching a dish collector (meters)
pub const COLLECTOR_ARRIVAL_EPS: Meters = 0.6;

/// Speed factor applied while the diner carries a tray
pub const CARRYING_TRAY_SPEED_FACTOR: f32 = 0.75;

/// Maximum time diners will keep searching for seats before giving up (seconds)
pub const MAX_SEAT_SEARCH_TIME_S: f32 = 45.0;

/// Default runtime dish values
pub const DEFAULT_DISH_QUANTITY: f32 = 100.0;
/// Quality in [0,1], where 1 is perfect and 0 is inedible
pub const DEFAULT_DISH_QUALITY: f32 = 0.8;
/// Contamination level in [0,1], where 0 is clean and 1 is hazardous
pub const DEFAULT_DISH_CONTAMINATION: f32 = 0.0;
/// Seconds since epoch or day start; 0 means "not set yet" for our sim
pub const DEFAULT_DISH_LAST_RESTOCKED_S: f32 = 0.0;
