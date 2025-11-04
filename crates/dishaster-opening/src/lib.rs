//! Opening animation simulation with flying dishes, emojis, and review texts

pub mod components;
pub mod resources;
mod sim;
mod systems;

pub use sim::{OpeningSimulationFeat, Simulation};

mod prelude {
    pub use dishrupt_core::prelude::*;
    pub use dishrupt_ecs::prelude::*;
    pub use dishrupt_rng::prelude::*;

    pub use crate::{components::*, resources::*};
}
