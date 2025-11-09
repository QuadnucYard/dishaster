use std::any::Any;

use dishaster_core::views::{CreditSectionView, CreditsView};
use dishaster_godot_opening::Opening;
use dishaster_godot_ui::{CreditsGui, StartMenuGui};
use dishaster_ui_protocol::AppRequest;
use dishrupt_godot::bind::BindGodot;
use dishrupt_godot_scene::{Scene, SceneContext, SceneId};
use godot::{classes::Node, global::godot_print, obj::Gd};

use crate::{game_main::CREDITS, scenes::proc::EnterLevelProcedure};

/// The root scene. Handles interaction outside levels.
pub struct StartScene {
    opening: Option<Opening>,

    gd: Gd<Node>,
}

impl BindGodot<Node> for StartScene {
    fn new(gd: Gd<Node>) -> Self {
        Self { opening: None, gd }
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
        ctx.gui.show::<StartMenuGui>();

        if self.opening.is_none() {
            let opening = Opening::new(self.gd.clone());
            self.opening = Some(opening);
        }
    }

    fn process(&mut self, ctx: &mut SceneContext, delta: f64) {
        if let Some(opening) = self.opening.as_mut() {
            opening.process(delta);
        };

        ctx.gui_cmds.run_cmds(ctx.gui);

        for req in ctx.gui_cmds.take_reqs() {
            let req: Box<dyn Any> = req;
            let req = req.downcast::<AppRequest>().expect("app request");

            self.handle_app_request(ctx, *req);
        }
    }
}

impl StartScene {
    fn handle_app_request(&mut self, ctx: &mut SceneContext, req: AppRequest) {
        match req {
            AppRequest::Quit => {
                godot_print!("Quit requested");
                self.gd.get_tree().expect("failed to get scene tree").quit();
            }
            AppRequest::EnterLevel => {
                ctx.schedule(EnterLevelProcedure);
            }
            AppRequest::ShowCredits => {
                let credits_data = CREDITS.get().expect("Credits should be available");

                let credits_view = CreditsView {
                    sections: credits_data
                        .sections
                        .iter()
                        .map(|s| CreditSectionView {
                            title: s.title.clone(),
                            entries: s.entries.clone(),
                        })
                        .collect(),
                };

                ctx.gui.hide::<StartMenuGui>();
                ctx.gui.get_mut::<CreditsGui>().set_view(credits_view);
                ctx.gui.show::<CreditsGui>();
            }
            AppRequest::BackToMenu => {
                ctx.gui.hide::<CreditsGui>();
                ctx.gui.show::<StartMenuGui>();
            }
            AppRequest::ExitLevel => panic!("should not happen in start menu"),
        }
    }
}
