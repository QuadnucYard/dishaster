use as_any::Downcast;
use dishrupt_godot_scene::*;

use crate::scenes::{GameScene, StartScene};

pub struct StartProcedure;

impl SceneProcedure for StartProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        // ctx.transition = Some(SceneTransition::Push(GameScene::ID));
        ctx.scene_stack.change_push_scene(ctx.base, StartScene::ID);

        SceneProcedurePoll::Ready
    }
}

pub struct EnterLevelProcedure();

impl SceneProcedure for EnterLevelProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        println!("enter level");

        ctx.scene_stack.change_push_scene(ctx.base, GameScene::ID);

        ctx.scene_stack.inspect_active_scene_mut(|scene| {
            let game_scene = (**scene).downcast_mut::<GameScene>().expect("game scene");

            game_scene.start_game();
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
