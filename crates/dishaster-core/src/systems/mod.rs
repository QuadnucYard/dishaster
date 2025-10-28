//! Bevy ECS systems for game logic processing

mod decision;
mod dining;
mod feedback;
mod monitor;
mod navigation;
mod queueing;
mod serving;
mod spawn;

pub use dining::*;
pub use monitor::*;
pub use navigation::*;
pub use queueing::*;
pub use serving::*;
pub use spawn::*;

/// Common imports for systems
mod prelude {
    pub use dishaster_channel::events::*;

    pub use crate::{components::*, constants::*, models::*, prelude::*, resources::*};
}
