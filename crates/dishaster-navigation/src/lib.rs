//! Navigation, pathfinding, and collision detection/resolution.

mod avoidance;
mod collision;
mod crowd;
mod grid;
mod path;
mod pathfinding;

mod prelude {
    pub use bevy_math::*;
}

pub use avoidance::*;
pub use collision::*;
pub use crowd::*;
pub use grid::*;
pub use path::*;
pub use pathfinding::*;
