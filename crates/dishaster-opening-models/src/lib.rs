//! Models for the Dishaster opening simulation.

mod credit;
mod opening;

pub use credit::{CreditSection, CreditsData};
pub use opening::{OpeningAssets, OpeningConfig, OpeningWorldConfig};

mod prelude {
    pub use dishrupt_core::prelude::*;
    pub use serde::Deserialize;
}
