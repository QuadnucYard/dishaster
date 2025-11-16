//! Models for the Dishaster opening simulation.

mod credit;
mod ending;
mod opening;

pub use credit::{CreditSection, CreditsData};
pub use ending::EndingModel;
pub use opening::{OpeningAssets, OpeningConfig, OpeningWorldConfig};

mod prelude {
    pub use dishrupt_core::prelude::*;
    pub use serde::Deserialize;
}
