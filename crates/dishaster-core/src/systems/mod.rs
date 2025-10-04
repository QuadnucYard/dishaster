//! Bevy ECS systems for game logic processing

mod dining;
mod feedback;
mod navigation;
mod queueing;
mod serving;
mod spawn;

pub use dining::*;
pub use feedback::*;
pub use navigation::*;
pub use queueing::*;
pub use serving::*;
pub use spawn::*;

/// Common imports for systems
mod prelude {
    pub use crate::{
        components::*, constants::*, models::*, prelude::*, resources::*, snapshots::*,
    };
}
