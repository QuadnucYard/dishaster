//! Opening animation simulation with flying foods, faces, and review texts

pub mod components;
pub mod protocol;
pub mod resources;
pub mod sim;
mod systems;

pub use resources::{OpeningAssets, OpeningConfig};
pub use sim::{OpeningSimulationFeat, Simulation};

mod prelude {
    pub use dishrupt_core::prelude::*;
    pub use dishrupt_ecs::prelude::*;
    pub use dishrupt_rng::prelude::*;

    pub use crate::{components::*, resources::*};
}
