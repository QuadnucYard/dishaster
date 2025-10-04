use as_any::Downcast;
use dishaster_godot_ui::{StartMenuUI, req::*};
use dishrupt_godot::bind::BindGodot;
use dishrupt_godot_scene::{Scene, SceneContext, SceneId};
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
        ctx.gui.show::<StartMenuUI>();
    }

    fn process(&mut self, ctx: &mut SceneContext, _delta: f64) {
        ctx.gui_cmds.run_cmds(ctx.gui);

        for req in ctx.gui_cmds.take_reqs() {
            let req = &*req;
            godot_print!("Got GUI request: {}", std::any::type_name_of_val(req));

            if req.is::<QuitRequest>() {
                godot_print!("Quit requested");
                self.gd.get_tree().unwrap().quit();
            }

            if req.is::<EnterLevelRequest>() {
                ctx.schedule(EnterLevelProcedure);
            }
        }
    }
}
