//! Opening animation simulation with flying foods, faces, and review texts

pub(crate) mod components;
pub mod protocol;
pub(crate) mod resources;
pub(crate) mod sim;
mod systems;

pub use resources::{OpeningAssetsRes, OpeningConfigRes};
pub use sim::{OpeningSimulationFeat, Simulation};

/// Re-export of dishaster_oepning_models
pub mod models {
    pub use dishaster_opening_models::*;
}

mod prelude {
    pub use dishrupt_core::prelude::*;
    pub use dishrupt_ecs::prelude::*;
    pub use dishrupt_rng::prelude::*;

    pub use crate::{components::*, resources::*};
}
