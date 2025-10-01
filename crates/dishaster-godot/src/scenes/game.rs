use std::cell::OnceCell;

use dishaster_godot_ui::*;
use dishrupt_godot::{bind::BindGodot, input::listener::GodotInputEvent};
use dishrupt_godot_scene::*;
use dishrupt_godot_ui::*;
use godot::{classes::Node, obj::Gd};

use crate::{game::Game, game_main::GAME_DATA};

/// The in-game scene.
pub struct GameScene {
    game: OnceCell<Game>,

    gd: Gd<Node>,
}

impl BindGodot<Node> for GameScene {
    fn new(gd: Gd<Node>) -> Self {
        Self {
            game: OnceCell::new(),

            gd,
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

        let gaming = gui.get_mut::<GamingLayout>();
        gaming.show();
        let time_stats = gui.get_mut::<TimeStatsGui>();
        time_stats.show();
    }

    fn process(&mut self, ctx: &mut SceneContext, delta: f64) {
        let game = self.game.get_mut().unwrap();

        game.process(delta, ctx);
    }

    fn input(&mut self, _ctx: &mut SceneContext, event: GodotInputEvent) {
        let game = self.game.get_mut().unwrap();

        game.process_input(event);
    }
}

impl GameScene {
    pub fn start_game(&mut self) {
        self.game.get_or_init(|| {
            let db = GAME_DATA.get().expect("game data not initialized");
            let level = db.levels.first().expect("first level unavailable").clone();
            Game::new(self.gd(), level)
        });
    }
}
