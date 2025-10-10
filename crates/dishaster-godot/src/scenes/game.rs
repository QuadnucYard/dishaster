use as_any::Downcast;
use dishaster_core::models::LevelConfig;
use dishaster_godot_ui::{req::*, *};
use dishrupt_godot::{bind::BindGodot, input::listener::GodotInputEvent};
use dishrupt_godot_scene::*;
use godot::{classes::Node, prelude::*};

use crate::{game::Game, scenes::proc::*};

/// The in-game scene. Recreate the inner `Game` instance on each run.
pub struct GameScene {
    game: Option<Game>,

    gd: Gd<Node>,
}

impl BindGodot<Node> for GameScene {
    fn new(gd: Gd<Node>) -> Self {
        Self { game: None, gd }
    }
}

impl GameScene {
    pub const ID: SceneId = "game";
}

impl Scene for GameScene {
    fn id(&self) -> SceneId {
        Self::ID
    }

    fn gd(&self) -> Gd<Node> {
        self.gd.clone()
    }

    fn can_cache(&self) -> bool {
        false
    }

    fn enter(&mut self, ctx: &mut SceneContext) {
        let gui = &mut ctx.gui;

        gui.show::<GamingLayout>();
        gui.show::<TimeStatsGui>();
    }

    fn process(&mut self, ctx: &mut SceneContext, delta: f64) {
        if let Some(game) = self.game.as_mut() {
            game.process(delta, ctx);
        };

        ctx.gui_cmds.run_cmds(ctx.gui);

        for req in ctx.gui_cmds.take_reqs() {
            let req = &*req;

            if let Some(game) = self.game.as_mut()
                && let Some(req) = req.downcast_ref::<SetTpsRequest>()
            {
                game.set_tps(ctx, req.0);
                continue;
            }

            if req.is::<StartRunRequest>()
                && let Some(game) = self.game.as_mut()
            {
                game.begin_run(ctx);
            }
            if req.is::<EndDayRequest>()
                && let Some(game) = self.game.as_mut()
            {
                game.force_finish_day(ctx);
            }
            if req.is::<NextDayRequest>() {
                ctx.schedule(AdvanceLevelProcedure);
            }
            if req.is::<ExitLevelRequest>() {
                ctx.schedule(ExitLevelProcedure);
            }
        }
    }

    fn input(&mut self, _ctx: &mut SceneContext, event: GodotInputEvent) {
        let Some(game) = self.game.as_mut() else {
            return;
        };

        game.process_input(event);
    }
}

impl GameScene {
    pub fn start_game(&mut self, ctx: &mut SceneContext, level: LevelConfig) {
        let mut game = Game::new(self.gd(), level);
        game.start_day(ctx);
        self.game = Some(game);
    }
}
