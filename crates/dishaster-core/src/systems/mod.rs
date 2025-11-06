//! Bevy ECS systems for game logic processing

mod decision;
mod dining;
mod feedback;
mod hint;
mod monitor;
mod navigation;
mod queueing;
mod refill;
mod serving;
mod spawn;

pub use dining::*;
pub use monitor::*;
pub use navigation::*;
pub use queueing::*;
pub use refill::*;
pub use serving::*;
pub use spawn::*;

/// Common imports for systems
mod prelude {
    pub use dishaster_interface::event::*;

    pub use crate::{
        components::*, constants::*, messages::*, models::*, prelude::*, resources::*,
    };
}
