//! Simulation-wide constants (all lengths are in meters)
//! Keep numbers centralized to avoid magic numbers scattered in logic.

use dishaster_models::Seconds;

use crate::models::Meters;

/// Spacing between customers standing in the same queue
pub const QUEUE_SPACING: Meters = 0.5;

/// Radius used when picking a wander/observation target
pub const WANDER_RADIUS: Meters = 2.0;

/// Forward offset (in +Y) from the window line to approach and order
/// Represents the distance a diner stands from the counter to order.
pub const WINDOW_APPROACH_OFFSET: Meters = 0.6;

/// Offset placing staff slightly behind the counter line
pub const WINDOW_STAFF_OFFSET: Meters = 0.3;

/// How close to an exit counts as leaving
pub const EXIT_ARRIVAL_EPS: Meters = 0.1;

/// Waypoint arrival tolerance when following a path
pub const PATH_WAYPOINT_EPS: Meters = 0.2;

/// Default per-diner initial satisfaction [0,1]
pub const DEFAULT_DINER_SATISFACTION: f32 = 0.5;

/// Minimum simulated delay for a diner to verbalize the order
pub const ORDER_SPEECH_DELAY_MIN: Seconds = 0.35;
/// Maximum simulated delay for a diner to verbalize the order
pub const ORDER_SPEECH_DELAY_MAX: Seconds = 0.85;
/// Minimum delay before staff confirms the order
pub const STAFF_CONFIRM_DELAY_MIN: Seconds = 0.6;
/// Maximum delay before staff confirms the order
pub const STAFF_CONFIRM_DELAY_MAX: Seconds = 1.4;
/// Random variation multiplier applied to dish serving time
pub const STAFF_SERVICE_TIME_VARIATION: f32 = 0.25;
/// Default walking speed for serving staff (meters per second)
pub const STAFF_WALK_SPEED: f32 = 1.25;
/// Navigation radius used for serving staff collision avoidance
pub const STAFF_COLLISION_RADIUS: Meters = 0.35;

/// Soft margin from canteen walls for free roaming movement
pub const DINING_AREA_MARGIN: Meters = 1.0;

/// How close counts as sitting down at a table
pub const TABLE_SEAT_ARRIVAL_EPS: Meters = 0.45;

/// Time a diner spends eating before leaving the table (seconds)
pub const BASE_EATING_DURATION_S: Seconds = 90.0;

/// Dirtiness increase applied to a table after a diner finishes eating
pub const TABLE_DIRTINESS_INCREMENT: f32 = 0.2;

/// Maximum allowed dirtiness value
pub const TABLE_MAX_DIRTINESS: f32 = 1.0;

/// Tolerance when reaching a dish collector
pub const COLLECTOR_ARRIVAL_EPS: Meters = 0.6;

/// Speed factor applied while the diner carries a tray
pub const CARRYING_TRAY_SPEED_FACTOR: f32 = 0.75;

/// Maximum time diners will keep searching for seats before giving up
pub const MAX_SEAT_SEARCH_TIME_S: Seconds = 45.0;

/// Default runtime dish values
pub const DEFAULT_DISH_QUANTITY: f32 = 100.0;
/// Quality in [0,1], where 1 is perfect and 0 is inedible
pub const DEFAULT_DISH_QUALITY: f32 = 0.8;
/// Contamination level in [0,1], where 0 is clean and 1 is hazardous
pub const DEFAULT_DISH_CONTAMINATION: f32 = 0.0;
/// Seconds since epoch or day start; 0 means "not set yet" for our sim
pub const DEFAULT_DISH_LAST_RESTOCKED_S: f32 = 0.0;
