//! Navigation, pathfinding, and collision detection/resolution.

mod collision;
mod crowd;
mod pathfinding;

mod prelude {
    pub use bevy_math::{IVec2, Rect, Vec2};
}

pub use collision::*;
pub use crowd::*;
pub use pathfinding::*;
