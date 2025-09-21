use crate::SceneProcedureContext;

pub enum SceneProcedurePoll {
    Ready,
    Pending,
}

pub trait SceneProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll;
}

pub struct PopSceneProcedure;

impl SceneProcedure for PopSceneProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        ctx.scene_stack.change_pop_scene(ctx.base);
        SceneProcedurePoll::Ready
    }
}
