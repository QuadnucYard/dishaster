use as_any::Downcast;
use dishaster_godot_ui::{StartMenuUI, req::*};
use dishrupt_godot::bind::BindGodot;
use dishrupt_godot_scene::{Scene, SceneContext, SceneId, SceneProcedure};
use dishrupt_godot_ui::*;
use godot::{classes::Node, global::godot_print, obj::Gd};

use crate::scenes::proc::details::EnterLevelProcedure;

/// The root scene. Handles interaction outside levels.
pub struct StartScene {
    gd: Gd<Node>,
}

impl BindGodot<Node> for StartScene {
    fn new(gd: Gd<Node>) -> Self {
        Self { gd }
    }
}

impl StartScene {
    pub const ID: SceneId = "start";
}

impl Scene for StartScene {
    fn id(&self) -> SceneId {
        Self::ID
    }

    fn gd(&self) -> Gd<Node> {
        self.gd.clone()
    }

    fn enter(&mut self, ctx: &mut SceneContext) {
        let start_menu = ctx.gui.get_mut::<StartMenuUI>();
        start_menu.show();
    }

    fn leave(&mut self, ctx: &mut SceneContext) {
        for gui in ctx.gui.iter_mut() {
            gui.set_active(false);
        }
    }

    fn process(&mut self, ctx: &mut SceneContext, _delta: f64) {
        ctx.gui_cmds.run_cmds(ctx.gui);
        let mut proc: Option<Box<dyn SceneProcedure>> = None;
        ctx.gui_cmds.run_reqs(|req| {
            let req = &*req;
            godot_print!("Got GUI request: {}", std::any::type_name_of_val(req));

            if let Some(_req) = req.downcast_ref::<QuitRequest>() {
                godot_print!("Quit requested");
                self.gd.get_tree().unwrap().quit();
            }

            if let Some(_req) = req.downcast_ref::<EnterLevelRequest>() {
                proc = Some(Box::new(EnterLevelProcedure {}));
            }
        });

        if let Some(proc) = proc {
            ctx.proc = Some(proc);
        }
    }
}
