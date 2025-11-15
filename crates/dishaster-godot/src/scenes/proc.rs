use dishaster_godot_game::persist::level_for_current_day;
use dishrupt_godot_scene::*;

use crate::{
    game_main::game_services,
    scenes::{GameScene, StartScene},
};

pub struct StartProcedure;

impl SceneProcedure for StartProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        // Initial scene load without transition
        ctx.scene_stack
            .change_push_scene_immediate(ctx.base, StartScene::ID);

        SceneProcedurePoll::Ready
    }
}

pub struct EnterLevelProcedure;

impl SceneProcedure for EnterLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        let services = game_services().clone();
        let level = level_for_current_day(&services.user_service.profiles, &services.data.models)
            .expect("failed to get current day level in progress store");

        // Use callback to initialize game scene after it's loaded
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
        ctx.scene_stack.change_push_scene_with_callback(
            ctx.base,
            GameScene::ID,
            Some(trans),
            move |scene, scene_ctx| {
                use std::any::Any;
                let game_scene = (scene as &mut dyn Any)
                    .downcast_mut::<GameScene>()
                    .expect("expected GameScene");
                game_scene.start_game(scene_ctx, level, services);
            },
        );

        SceneProcedurePoll::Ready
    }
}

pub struct ExitLevelProcedure;

impl SceneProcedure for ExitLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
        ctx.scene_stack.change_pop_scene(ctx.base, Some(trans));

        SceneProcedurePoll::Ready
    }
}

pub struct AdvanceLevelProcedure;

impl SceneProcedure for AdvanceLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
        ctx.scene_stack.change_pop_scene(ctx.base, Some(trans)); // pop GameScene

        EnterLevelProcedure.process(ctx) // push new GameScene
    }
}
