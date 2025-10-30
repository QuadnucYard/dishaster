use as_any::Downcast;
use dishaster_core::{models::LevelConfig, sim::Simulation};
use dishaster_godot_game::Game;
use dishaster_godot_ui::{req::*, *};
use dishrupt_godot::{bind::BindGodot, input::listener::GodotInputEvent};
use dishrupt_godot_scene::*;
use godot::{classes::Node, prelude::*};

use crate::scenes::proc::*;

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

            if let Some(req) = req.downcast_ref::<GameRequest>() {
                if let Some(game) = self.game.as_mut() {
                    match req {
                        GameRequest::NextDay => {
                            // This is handled specially to allow for scheduling
                            ctx.schedule(AdvanceLevelProcedure)
                        }
                        _ => game.handle_request(ctx, req),
                    }
                }
                continue;
            }

            let req = req.downcast_ref::<AppRequest>().expect("app request");

            match *req {
                AppRequest::Quit | AppRequest::EnterLevel => {
                    panic!("should be handled in main menu")
                }
                AppRequest::ExitLevel => {
                    ctx.schedule(ExitLevelProcedure);
                }
            }
        }
    }

    fn input(&mut self, ctx: &mut SceneContext, event: GodotInputEvent) {
        let Some(game) = self.game.as_mut() else {
            return;
        };

        game.process_input(ctx, event);
    }
}

impl GameScene {
    pub fn start_game(&mut self, ctx: &mut SceneContext, level: LevelConfig) {
        let mut game = Game::new(self.gd(), level, |db, level| {
            let mut sim = Box::new(Simulation::new(db));
            sim.start(level);
            sim
        });
        game.start_day(ctx);
        self.game = Some(game);
    }
}
