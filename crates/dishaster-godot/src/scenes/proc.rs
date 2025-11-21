use dishaster_godot_game::persist::level_for_current_day;
use dishrupt_godot_scene::*;

use crate::{
    prelude::*,
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
        let (data, user_service) = ctx.res.get_many::<(GameDataAssets, UserDataService)>();

        let level = level_for_current_day(&user_service.profiles, &data.models)
            .expect("failed to get current day level in progress store");

        // Use callback to initialize game scene after it's loaded
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
        ctx.scene_stack.change_push_scene_with_callback(
            GameScene::ID,
            Some(trans),
            move |scene, scene_ctx| {
                use std::any::Any;
                let game_scene = (scene as &mut dyn Any)
                    .downcast_mut::<GameScene>()
                    .expect("expected GameScene");
                game_scene.start_game(scene_ctx, level);
            },
        );

        SceneProcedurePoll::Ready
    }
}

pub struct ExitLevelProcedure;

impl SceneProcedure for ExitLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
        ctx.scene_stack.change_pop_scene(Some(trans));

        SceneProcedurePoll::Ready
    }
}

pub struct AdvanceLevelProcedure;

impl SceneProcedure for AdvanceLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        let (data, user_service) = ctx.res.get_many::<(GameDataAssets, UserDataService)>();

        let level = level_for_current_day(&user_service.profiles, &data.models)
            .expect("failed to get current day level in progress store");

        // Replace current GameScene with new GameScene (no pop to main menu)
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
        ctx.scene_stack.replace_scene_with_callback(
            GameScene::ID,
            Some(trans),
            move |scene, scene_ctx| {
                use std::any::Any;
                let game_scene = (scene as &mut dyn Any)
                    .downcast_mut::<GameScene>()
                    .expect("expected GameScene");
                game_scene.start_game(scene_ctx, level);
            },
        );

        SceneProcedurePoll::Ready
    }
}
