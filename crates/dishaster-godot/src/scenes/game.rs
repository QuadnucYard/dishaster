use as_any::Downcast;
use dishaster_godot_ui::{req::*, *};
use dishrupt_godot::{bind::BindGodot, input::listener::GodotInputEvent};
use dishrupt_godot_scene::*;
use godot::{classes::Node, global::godot_print, obj::Gd};

use crate::{game::Game, game_main::GAME_DATA, scenes::proc::details::ExitLevelProcedure};

/// The in-game scene. Recreate the inner `Game` instance on each run.
pub struct GameScene {
    game: Option<Game>,

    gd: Gd<Node>,

    /// How many days we've played in this session, used to offset level day/seed.
    day_offset: u32,
}

impl BindGodot<Node> for GameScene {
    fn new(gd: Gd<Node>) -> Self {
        Self {
            game: None,
            gd,

            day_offset: 0,
        }
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
            godot_print!("Got GUI request: {}", std::any::type_name_of_val(req));

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
                // self.start_new_day(ctx);
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
    pub fn start_game(&mut self, ctx: &mut SceneContext) {
        let mut game = {
            let db = GAME_DATA.get().expect("game data not initialized");
            let level = db.levels.first().expect("first level unavailable").clone();
            Game::new(self.gd(), level)
        };
        game.start_day(ctx);
        self.game = Some(game);
    }

    pub fn start_new_day(&mut self, ctx: &mut SceneContext) {
        // todo: we should recreate the game scene.
        self.day_offset += 1;
        let mut game = {
            let db = GAME_DATA.get().expect("game data not initialized");
            let mut level = db.levels.first().expect("first level unavailable").clone();
            level.day += self.day_offset;
            level.seed += self.day_offset as u64; // Change seed each day for variety
            Game::new(self.gd(), level)
        };
        game.start_day(ctx);
        self.game = Some(game);
    }
}
