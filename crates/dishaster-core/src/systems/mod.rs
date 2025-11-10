//! Bevy ECS systems for game logic processing

mod decision;
mod dining;
mod feedback;
mod hint;
mod lifecycle;
mod management;
mod monitor;
mod navigation;
mod persist;
mod queueing;
mod refill;
mod serving;
mod spawn;

pub use dining::*;
pub use feedback::*;
pub use lifecycle::*;
pub use management::*;
pub use monitor::*;
pub use navigation::*;
pub use persist::*;
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
