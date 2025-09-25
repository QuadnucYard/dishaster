//! Game model definitions and data structures

mod canteen;
mod diner;
mod dish;
mod level;
mod movement;

pub use canteen::*;
pub use diner::*;
pub use dish::*;
pub use level::*;
pub use movement::*;
pub use prelude::ModelId;
use serde::Deserialize;

mod prelude {
    pub use serde::{Deserialize, Serialize};

    pub use super::{Meters, MinMax, Seconds, Size, XRange};
    pub use crate::prelude::*;
}

/// Physical distance measurement in meters
///
/// Used for all spatial calculations including object sizes,
/// positions, and movement distances within the simulation.
pub type Meters = f32;

/// Time duration measurement in seconds
///
/// Used for timing calculations, delays, and duration-based
/// game mechanics throughout the simulation.
pub type Seconds = f32;
