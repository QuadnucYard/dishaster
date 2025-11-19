use dishaster_interface::{snapshots::DebugFlags, *};
use dishaster_views::PricingMethod;
use dishrupt_core::EntityId;

use crate::Game;

impl Game {
    /// Update the simulation tick rate and refresh related UI state.
    pub fn set_tps(&mut self, requested_tps: f32) {
        self.sim_runner.set_tps(requested_tps as f64);
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_enabled = debug_mode;

        self.send_sim_command(SimCommand::SetDebugFlags(if debug_mode {
            DebugFlags::all()
        } else {
            DebugFlags::none()
        }));

        self.dbgviz.set_visible(debug_mode);

        for agent in self.pres.agents.values_mut() {
            agent.set_debug_enabled(debug_mode);
        }
    }

    pub fn set_dish_price(&mut self, dish: EntityId, pricing: PricingMethod) {
        self.send_sim_command(SimCommand::UpdateDishPricing {
            dish_entity: dish,
            pricing,
        });

        // The UI will be updated via the SimEvent::DishPriceChanged event
    }
}
