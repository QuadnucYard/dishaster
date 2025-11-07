//! Channel definitions for communicating with the Dishaster simulation.

pub mod command;
pub mod event;
pub mod query;
pub mod response;
pub mod snapshots;

use dishaster_save_models::SimProfile;
use dishrupt_simulation::SimulationFeature;

pub use crate::{
    command::SimCommand, event::SimEvent, query::SimQuery, response::SimResponse,
    snapshots::Snapshot,
};

/// Core simulation feature definition
pub struct CoreSimulationFeat;

impl SimulationFeature for CoreSimulationFeat {
    type Snapshot = Snapshot;
    type Command = SimCommand;
    type Query = SimQuery;
    type Event = SimEvent;
    type Response = SimResponse;
    type Profile = SimProfile;
}
