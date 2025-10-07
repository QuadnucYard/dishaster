use as_any::Downcast;
use dishrupt_godot_scene::*;

use crate::{
    game_main::progress_service,
    scenes::{GameScene, StartScene},
};

pub struct StartProcedure;

impl SceneProcedure for StartProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        // ctx.transition = Some(SceneTransition::Push(GameScene::ID));
        ctx.scene_stack.change_push_scene(ctx.base, StartScene::ID);

        SceneProcedurePoll::Ready
    }
}

pub struct EnterLevelProcedure;

impl SceneProcedure for EnterLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        println!("enter level");

        let level = {
            let service = progress_service();
            service
                .level_for_current_day()
                .expect("failed to get current day level in progress store")
        };

        ctx.scene_stack.change_push_scene(ctx.base, GameScene::ID);

        ctx.scene_stack.inspect_active_scene_mut(|scene| {
            let game_scene = scene.downcast_mut::<GameScene>().expect("game scene");

            game_scene.start_game(ctx.base, level);
        });

        SceneProcedurePoll::Ready
    }
}

pub struct ExitLevelProcedure;

impl SceneProcedure for ExitLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        // assert!(ctx.scene_manager.is_active_of_type::<GameScene>());
        ctx.scene_stack.change_pop_scene(ctx.base);
        SceneProcedurePoll::Ready
    }
}

pub struct AdvanceLevelProcedure;

impl SceneProcedure for AdvanceLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        ctx.scene_stack.change_pop_scene(ctx.base); // pop GameScene

        EnterLevelProcedure.process(ctx) // push new GameScene
    }
}
