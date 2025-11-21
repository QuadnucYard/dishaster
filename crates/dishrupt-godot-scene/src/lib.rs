//! Scene management system for Godot-based games.
//!
//! This crate provides a flexible scene stack system with lifecycle management,
//! transition effects, and procedure-based scene changes. It abstracts the complexity
//! of managing multiple scenes, their loading/unloading, and smooth transitions between them.
//!
//! # Core Concepts
//!
//! - **Scene**: A game state (menu, gameplay, settings, etc.) with lifecycle hooks
//! - **SceneStack**: Manages the active scene and history, handles push/pop operations
//! - **SceneManager**: Top-level coordinator for scenes, procedures, and transitions
//! - **SceneProcedure**: Async-like operations for scene changes (enter level, advance, etc.)
//! - **SceneTransition**: Trait for visual transition effects (fade, slide, etc.)
//!
//! # Example
//!
//! ```rust,ignore
//! // Define a scene
//! struct MenuScene {
//!     gd: Gd<Node>,
//! }
//!
//! impl Scene for MenuScene {
//!     fn id(&self) -> SceneId { "menu" }
//!     fn gd(&self) -> Gd<Node> { self.gd.clone() }
//!
//!     fn enter(&mut self, ctx: &mut SceneContext) {
//!         // Setup UI, start music, etc.
//!     }
//!
//!     fn leave(&mut self, ctx: &mut SceneContext) {
//!         // Cleanup
//!     }
//! }
//!
//! // Schedule a scene change
//! ctx.schedule(EnterGameProcedure);
//! ```

mod manager;
mod proc;
mod resource;
mod transition;

use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use dishrupt_godot_input::event::GodotInputEvent;
use godot::{classes::Node, obj::Gd};

pub use crate::{manager::*, proc::*, resource::SceneResources, transition::*};

/// Static string identifier for scene types.
///
/// Each scene implementation should have a unique constant ID that identifies it.
/// This is used for scene loading, caching, and history tracking.
pub type SceneId = &'static str;

/// Context passed to scene lifecycle methods and procedures.
///
/// Provides access to game systems that scenes need to interact with:
/// - UI registry for showing/hiding GUI elements
/// - Audio manager for playing sounds and music
/// - Scene root node for creating transitions
/// - Ability to schedule scene procedures
pub struct SceneContext<'a> {
    /// Resources available to the scene
    pub res: &'a mut SceneResources,
    /// Currently scheduled scene procedure (if any)
    pub proc: Option<Box<dyn SceneProcedure>>,
}

impl SceneContext<'_> {
    /// Schedule a scene procedure to run in the next frame.
    ///
    /// This is the primary way to trigger scene changes. The procedure will be
    /// executed by the `SceneManager` during its `process()` call.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// ctx.schedule(EnterLevelProcedure);
    /// ```
    pub fn schedule(&mut self, f: impl SceneProcedure + 'static) {
        self.proc.replace(Box::new(f));
    }
}

/// Extended context for scene procedures.
///
/// Provides access to the scene stack and scene root node in addition to
/// the base `SceneContext`. This allows procedures to trigger scene changes
/// with custom transitions.
pub struct SceneProcedureContext<'a> {
    /// Base scene context with access to game systems
    pub base: &'a mut SceneContext<'a>,
    /// Scene stack for managing scene changes
    pub scene_stack: &'a mut SceneStack,
    /// Root node where scenes are attached (for creating transitions)
    pub scene_root: Gd<Node>,
}

// Deref implementations allow procedures to access SceneContext methods directly
impl<'a> Deref for SceneProcedureContext<'a> {
    type Target = SceneContext<'a>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl DerefMut for SceneProcedureContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

/// Scene transition type (legacy enum, kept for compatibility).
///
/// **Note**: Direct use of `SceneStack` methods is now preferred over this enum.
#[deprecated(
    since = "0.1.0",
    note = "Use SceneStack methods directly (change_push_scene, change_pop_scene, etc.)"
)]
pub enum SceneTransitionType {
    /// Push a new scene onto the stack
    Push(SceneId),
    /// Replace current scene with a new one
    Replace(SceneId),
    /// Pop current scene and return to previous
    Pop,
}

/// Core trait for game scenes.
///
/// Scenes represent distinct game states (menus, gameplay, settings, etc.) with
/// their own UI, logic, and lifecycle. Each scene must provide:
///
/// - A unique ID for identification
/// - A Godot node for rendering
/// - Lifecycle hooks for setup, teardown, and updates
///
/// # Lifecycle
///
/// 1. **ready()** - Called once when scene is first loaded
/// 2. **enter()** - Called when scene becomes active (may be called multiple times if cached)
/// 3. **process()/physics_process()** - Called every frame while active
/// 4. **input()** - Called for input events while active
/// 5. **leave()** - Called when scene is deactivated (hides UI by default)
///
/// # Caching
///
/// Scenes can opt into caching via `can_cache()`. Cached scenes remain in memory
/// when popped from the stack, allowing instant reactivation without reloading.
#[allow(unused)]
pub trait Scene: Any {
    /// Unique identifier for this scene type.
    fn id(&self) -> SceneId;

    /// Godot node representing this scene's visual hierarchy.
    fn gd(&self) -> Gd<Node>;

    /// Whether the scene should be cached when unloaded.
    ///
    /// Cached scenes remain in memory and can be quickly reactivated.
    /// Non-cached scenes are freed (queue_free) when removed from the stack.
    ///
    /// Default: `true` (scenes are cached)
    fn can_cache(&self) -> bool {
        true
    }

    /// Called once when the scene is first loaded.
    ///
    /// Use this for one-time initialization that should not be repeated
    /// even if the scene is cached and re-entered.
    fn ready(&mut self, ctx: &mut SceneContext) {}

    /// Called when the scene becomes active.
    ///
    /// This is called both on initial load (after `ready()`) and when
    /// returning to a cached scene. Use this to show UI, start music, etc.
    fn enter(&mut self, ctx: &mut SceneContext) {}

    /// Called when the scene is deactivated.
    ///
    /// Override to add custom cleanup logic (stop music, save state, etc.).
    fn leave(&mut self, ctx: &mut SceneContext) {}

    /// Called every frame while the scene is active.
    ///
    /// # Parameters
    /// - `delta`: Time elapsed since last frame in seconds
    fn process(&mut self, ctx: &mut SceneContext, delta: f64) {}

    /// Called every physics frame while the scene is active.
    ///
    /// Physics updates run at a fixed rate (typically 60 Hz).
    ///
    /// # Parameters
    /// - `delta`: Physics timestep in seconds (usually 1/60)
    fn physics_process(&mut self, ctx: &mut SceneContext, delta: f64) {}

    /// Called for input events while the scene is active.
    fn input(&mut self, ctx: &mut SceneContext, event: GodotInputEvent) {}
}
