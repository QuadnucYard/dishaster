//! Scene procedure system for async-like scene operations.
//!
//! Procedures provide a way to execute multi-step scene changes that may span
//! multiple frames, similar to async tasks. They can trigger transitions, wait
//! for them to complete, and coordinate complex scene changes.

use crate::{SceneProcedureContext, transition::FadeTransition};

/// Return type for scene procedure processing.
///
/// Indicates whether the procedure has completed or needs more frames to finish.
pub enum SceneProcedurePoll {
    /// Procedure has completed and should be removed
    Ready,
    /// Procedure needs more processing time (will be called again next frame)
    Pending,
}

/// Trait for scene change procedures.
///
/// Procedures are executed by the `SceneManager` and can perform complex
/// scene changes that require coordination with transitions and the scene stack.
///
/// # Examples
///
/// ```rust,ignore
/// struct EnterGameProcedure;
///
/// impl SceneProcedure for EnterGameProcedure {
///     fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
///         ctx.scene_stack.change_push_scene_with_callback(
///             ctx.base,
///             GameScene::ID,
///             ctx.transition,
///             |scene, ctx| {
///                 // Initialize game after scene loads
///                 let game = scene.downcast_mut::<GameScene>().unwrap();
///                 game.start_level(ctx);
///             }
///         );
///         SceneProcedurePoll::Ready
///     }
/// }
/// ```
pub trait SceneProcedure {
    /// Process one step of the procedure.
    ///
    /// Return `Ready` when done, or `Pending` to be called again next frame.
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll;
}

/// Simple procedure to pop the current scene and return to the previous one.
///
/// This triggers a fade-out transition, pops the scene, then fades back in.
/// Uses the default `FadeTransition`.
pub struct PopSceneProcedure;

impl SceneProcedure for PopSceneProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
        ctx.scene_stack.change_pop_scene(ctx.base, Some(trans));
        SceneProcedurePoll::Ready
    }
}
