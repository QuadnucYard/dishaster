use dishaster_channel::{commands::SimCommand, snapshots::DebugFlags};
use dishaster_godot_ui::{req::GameRequest, *};
use dishaster_models::PricingMethod;
use dishrupt_core::EntityId;
use dishrupt_godot_scene::SceneContext;

use crate::Game;

impl Game {
    pub fn handle_request(&mut self, ctx: &mut SceneContext, req: &GameRequest) {
        match *req {
            GameRequest::StartRun => {
                self.begin_run(ctx);
            }
            GameRequest::EndRun => {
                self.force_finish_day(ctx);
            }
            GameRequest::NextDay => unreachable!("handled specially in game scene"),
            GameRequest::SetTps(tps) => {
                self.set_tps(ctx, tps);
            }
            GameRequest::SetDebugMode(mode) => {
                self.set_debug_mode(mode);
            }

            GameRequest::ApplyDishPrice { dish, method } => {
                self.set_dish_price(dish, method);
            }
        }
    }

    /// Update the simulation tick rate and refresh related UI state.
    fn set_tps(&mut self, ctx: &mut SceneContext, requested_tps: f32) {
        if (self.sim_runner.tps() - requested_tps as f64).abs() <= f64::EPSILON {
            return;
        }

        self.sim_runner.set_tps(requested_tps as f64);

        ctx.gui
            .get_mut::<TimeStatsGui>()
            .set_tps_display(requested_tps);
    }

    fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_enabled = debug_mode;

        self.send_sim_command(SimCommand::SetDebugFlags(if debug_mode {
            DebugFlags::all()
        } else {
            DebugFlags::none()
        }));

        self.dbgviz.distance_overlay.set_visible(debug_mode);
        self.dbgviz.movement_overlay.set_visible(debug_mode);

        for agent in self.dc.agents.values_mut() {
            agent.set_debug_enabled(debug_mode);
        }
    }

    fn set_dish_price(&mut self, dish: EntityId, pricing: PricingMethod) {
        self.send_sim_command(SimCommand::UpdateDishPricing {
            dish_entity: dish,
            pricing,
        });

        if let Some(controller) = self.dc.dishes.get_mut(&dish) {
            controller.set_price(pricing);
        }
    }
}
