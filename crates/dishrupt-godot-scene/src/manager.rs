//! Scene management and stack implementation.
//!
//! This module provides the core scene management infrastructure:
//! - `SceneManager`: High-level coordinator for scenes, procedures, and transitions
//! - `SceneStack`: Stack-based scene history with push/pop operations
//! - `SceneLoader`: Trait for loading scene instances by ID

use std::collections::HashMap;

use godot::{classes::Node, obj::Gd};

use crate::{
    Scene, SceneContext, SceneId, SceneProcedureContext, SceneTransition,
    proc::{SceneProcedure, SceneProcedurePoll},
};

/// Trait for loading scene instances by their ID.
///
/// Implement this to provide scene construction logic. Typically checks the ID
/// and constructs the appropriate scene type.
///
/// # Examples
///
/// ```rust,ignore
/// struct MySceneLoader;
///
/// impl SceneLoader for MySceneLoader {
///     fn load(&self, id: SceneId) -> Box<dyn Scene> {
///         match id {
///             "menu" => Box::new(MenuScene::new()),
///             "game" => Box::new(GameScene::new()),
///             _ => panic!("Unknown scene: {}", id),
///         }
///     }
/// }
/// ```
pub trait SceneLoader {
    /// Load and construct a scene instance for the given ID.
    fn load(&self, id: SceneId) -> Box<dyn Scene>;
}

/// Callback type for scene initialization after loading.
///
/// Callbacks receive mutable access to the newly loaded scene and scene context,
/// allowing initialization code to run after a scene is loaded during a transition.
type SceneLoadedCallback = Box<dyn FnOnce(&mut dyn Scene, &mut SceneContext) + Send>;

/// Top-level scene manager coordinating scenes, procedures, and transitions.
///
/// The `SceneManager` is the main entry point for scene management. It:
/// - Owns the scene stack
/// - Processes scene procedures
/// - Coordinates scene changes
/// - Delegates scene lifecycle events
///
/// Transitions are provided per scene change, allowing different effects
/// for different transitions.
///
/// # Usage
///
/// Create once at game startup and call `process()` every frame:
///
/// ```rust,ignore
/// let mut manager = SceneManager::new(scene_root, MySceneLoader);
///
/// // In game loop:
/// manager.process(&mut scene_ctx);
/// ```
pub struct SceneManager {
    /// Scene stack managing active/cached scenes
    stack: SceneStack,
    /// Currently executing procedure (if any)
    proc: Option<Box<dyn SceneProcedure>>,
    /// Active transition (only present during scene changes)
    transition: Option<Box<dyn SceneTransition>>,
}

impl SceneManager {
    /// Create a new scene manager.
    ///
    /// # Parameters
    /// - `scene_root`: Godot node where scenes will be added as children
    /// - `scene_loader`: Implementation of scene loading logic
    pub fn new(scene_root: Gd<Node>, scene_loader: impl SceneLoader + 'static) -> Self {
        Self {
            stack: SceneStack::new(scene_root, scene_loader),
            proc: Default::default(),
            transition: None,
        }
    }

    /// Get a reference to the currently active scene.
    #[must_use]
    pub fn active_scene(&self) -> &Option<Box<dyn Scene>> {
        &self.stack.active_scene
    }

    /// Get a mutable reference to the currently active scene.
    pub fn active_scene_mut(&mut self) -> &mut Option<Box<dyn Scene>> {
        &mut self.stack.active_scene
    }

    /// Inspect the active scene with a read-only closure.
    pub fn inspect_active_scene(&self, f: impl FnOnce(&dyn Scene)) {
        self.stack.inspect_active_scene(f);
    }

    /// Inspect the active scene with a mutable closure.
    pub fn inspect_active_scene_mut(&mut self, f: impl FnOnce(&mut dyn Scene)) {
        self.stack.inspect_active_scene_mut(f);
    }

    /// Get a clone of the scene root node.
    ///
    /// This node is the parent for all scene instances managed by this manager.
    #[must_use]
    pub fn scene_root(&self) -> Gd<Node> {
        self.stack.scene_root()
    }

    /// Schedule a scene procedure to run starting next frame.
    ///
    /// Only one procedure can be active at a time. Scheduling a new procedure
    /// replaces any existing one.
    pub fn schedule(&mut self, f: impl SceneProcedure + 'static) {
        self.proc.replace(Box::new(f));
    }

    /// Schedule a boxed scene procedure (for dynamic dispatch).
    pub fn schedule1(&mut self, proc: Box<dyn SceneProcedure>) {
        self.proc.replace(proc);
    }

    /// Process scene management for this frame.
    ///
    /// This should be called every frame from your main game loop. It:
    /// 1. Starts transitions for pending scene changes
    /// 2. Updates transition animations (if active)
    /// 3. Processes pending scene changes (when transition-out completes)
    /// 4. Runs the active scene procedure (if any)
    pub fn process<'a>(&'a mut self, ctx: &'a mut SceneContext<'a>) {
        // Check if we need to start a transition for a pending change
        if self.transition.is_none()
            && self.stack.has_pending_change()
            && let Some(mut trans) = self.stack.take_pending_transition()
        {
            trans.transition_out(None);
            self.transition = Some(trans);
        }

        // Update transition if active
        if let Some(transition) = &mut self.transition {
            transition.process();

            // Process pending scene changes if transition-out is complete
            if !transition.is_transitioning() {
                self.stack.process_pending_change(ctx, &mut **transition);
                // Transition will continue with transition_in, keep it active
            }
        }

        if let Some(mut proc) = self.proc.take() {
            log::debug!("Processing scene procedure");
            let scene_root = self.stack.scene_root();
            let ctx = &mut SceneProcedureContext {
                base: ctx,
                scene_stack: &mut self.stack,
                scene_root,
            };
            let r = proc.process(ctx);
            if matches!(r, SceneProcedurePoll::Pending) {
                self.proc.replace(proc);
            }
        }

        // Clear transition after transition-in completes
        if let Some(transition) = &self.transition
            && !transition.is_transitioning()
        {
            self.transition = None;
        }
    }
}

/// Stack-based scene management with history and caching.
///
/// The `SceneStack` maintains:
/// - An active scene (currently visible and processing)
/// - A history of scene IDs (for back navigation)
/// - A cache of unloaded but cacheable scenes (for fast reactivation)
///
/// # Scene Changes
///
/// - **Push**: Add a new scene on top (current scene hidden and possibly cached)
/// - **Pop**: Remove current scene and restore previous from history
/// - **Replace**: Swap current scene with a new one (no history change)
///
/// # Transitions
///
/// Scene changes normally happen with transitions:
/// 1. Fade out (scene still active)
/// 2. Hide old scene, load new scene (happens when fade completes)
/// 3. Fade in (new scene now active)
///
/// Use `_immediate` variants for instant changes without transitions.
pub struct SceneStack {
    /// Currently active scene (visible and processing)
    active_scene: Option<Box<dyn Scene>>,
    /// Cached scenes that can be quickly reactivated
    unloaded_scenes: HashMap<SceneId, Box<dyn Scene>>,
    /// History of scene IDs for back navigation
    history: Vec<SceneId>,

    /// Godot node where scenes are added as children
    scene_root: Gd<Node>,
    /// Scene loader for instantiating new scenes
    scene_loader: Box<dyn SceneLoader>,

    /// Pending scene change waiting for transition to complete
    pending_change: Option<PendingSceneChange>,
}

/// Pending scene change waiting for transition to complete.
struct PendingSceneChange {
    /// Type of scene change operation
    change_type: ChangeType,
    /// Optional callback to execute after scene is loaded and active
    on_loaded: Option<SceneLoadedCallback>,
    /// Transition storage
    transition: Option<Box<dyn SceneTransition>>,
}

/// Type of scene stack operation.
enum ChangeType {
    /// Push a new scene onto the stack
    Push(SceneId),
    /// Pop the current scene
    Pop,
    /// Replace the current scene
    Replace(SceneId),
}

impl SceneStack {
    /// Create a new scene stack.
    ///
    /// # Parameters
    /// - `scene_root`: Godot node where scenes will be added
    /// - `scene_loader`: Implementation for loading scenes by ID
    pub fn new(scene_root: Gd<Node>, scene_loader: impl SceneLoader + 'static) -> Self {
        Self {
            scene_root,
            active_scene: Default::default(),
            unloaded_scenes: Default::default(),
            history: Default::default(),
            pending_change: None,

            scene_loader: Box::new(scene_loader),
        }
    }

    /// Inspect the active scene with a read-only closure.
    pub fn inspect_active_scene(&self, f: impl FnOnce(&dyn Scene)) {
        self.active_scene.as_ref().inspect(|&scene| f(&**scene));
    }

    /// Inspect the active scene with a mutable closure.
    pub fn inspect_active_scene_mut(&mut self, f: impl FnOnce(&mut dyn Scene)) {
        if let Some(scene) = self.active_scene.as_mut() {
            f(&mut **scene);
        }
    }

    /// Get a reference to the scene root node.
    ///
    /// This node is the parent for all scene instances managed by this stack.
    #[must_use]
    pub fn scene_root(&self) -> Gd<Node> {
        self.scene_root.clone()
    }

    /// Check if there is a pending scene change.
    pub fn has_pending_change(&self) -> bool {
        self.pending_change.is_some()
    }

    /// Take the transition from the pending change without consuming the whole change.
    ///
    /// This allows SceneManager to start the transition-out while keeping the
    /// pending change details for later processing.
    pub(crate) fn take_pending_transition(&mut self) -> Option<Box<dyn SceneTransition>> {
        self.pending_change.as_mut()?.transition.take()
    }

    /// Push a new scene onto the stack with transition.
    ///
    /// The current scene is hidden and cached (if cacheable), then the new scene
    /// is loaded and shown after the transition-out completes.
    ///
    /// # Parameters
    /// - `scene_id`: ID of the scene to load and push
    /// - `transition`: Optional transition effect to use for this scene change
    pub fn change_push_scene(
        &mut self,
        scene_id: SceneId,
        transition: Option<Box<dyn SceneTransition>>,
    ) {
        // Warn if overwriting a pending change (likely a bug)
        if self.pending_change.is_some() {
            log::warn!(
                target: "scene",
                "Overwriting pending scene change with push to {:?} - this may cause issues",
                scene_id
            );
        }
        // Store the pending scene change
        self.pending_change = Some(PendingSceneChange {
            change_type: ChangeType::Push(scene_id),
            on_loaded: None,
            transition,
        });
    }

    /// Push a new scene with transition and post-load callback.
    ///
    /// Like `change_push_scene`, but executes a callback after the scene is loaded.
    /// This is useful for initializing the scene with specific data or state.
    ///
    /// # Parameters
    /// - `scene_id`: ID of the scene to load
    /// - `transition`: Optional transition effect to use for this scene change
    /// - `on_loaded`: Callback receiving mutable scene and context after loading
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));
    /// ctx.scene_stack.change_push_scene_with_callback(
    ///     GameScene::ID,
    ///     Some(trans),
    ///     move |scene, ctx| {
    ///         let game = scene.downcast_mut::<GameScene>().unwrap();
    ///         game.start_level(ctx, level_data);
    ///     }
    /// );
    /// ```
    pub fn change_push_scene_with_callback(
        &mut self,
        scene_id: SceneId,
        transition: Option<Box<dyn SceneTransition>>,
        on_loaded: impl FnOnce(&mut dyn Scene, &mut SceneContext) + Send + 'static,
    ) {
        // Warn if overwriting a pending change (likely a bug)
        if self.pending_change.is_some() {
            log::warn!(
                target: "scene",
                "Overwriting pending scene change with push to {:?} - this may cause issues",
                scene_id
            );
        }
        // Store the pending scene change with callback
        self.pending_change = Some(PendingSceneChange {
            change_type: ChangeType::Push(scene_id),
            on_loaded: Some(Box::new(on_loaded)),
            transition,
        });
    }

    /// Pop the current scene and return to the previous one with transition.
    ///
    /// The current scene is removed from the stack, and the previous scene from
    /// history is restored and shown after the transition-out completes.
    ///
    /// # Parameters
    /// - `transition`: Optional transition effect to use for this scene change
    ///
    /// # Panics
    /// Panics if there is no previous scene in the history (i.e., popping the root scene).
    pub fn change_pop_scene(&mut self, transition: Option<Box<dyn SceneTransition>>) {
        // Warn if overwriting a pending change (likely a bug)
        if self.pending_change.is_some() {
            log::warn!(
                target: "scene",
                "Overwriting pending scene change with pop - this may cause issues"
            );
        }
        // Store the pending scene change
        self.pending_change = Some(PendingSceneChange {
            change_type: ChangeType::Pop,
            on_loaded: None,
            transition,
        });
    }

    /// Replace the current scene with a new one with transition.
    ///
    /// Unlike push, this does not add to the history. The old scene is removed
    /// and the new scene takes its place in the stack.
    ///
    /// # Parameters
    /// - `scene_id`: ID of the scene to load as replacement
    /// - `transition`: Optional transition effect to use for this scene change
    pub fn replace_scene(
        &mut self,
        scene_id: SceneId,
        transition: Option<Box<dyn SceneTransition>>,
    ) {
        // Warn if overwriting a pending change (likely a bug)
        if self.pending_change.is_some() {
            log::warn!(
                target: "scene",
                "Overwriting pending scene change with replace to {:?} - this may cause issues",
                scene_id
            );
        }
        // Store the pending scene change
        self.pending_change = Some(PendingSceneChange {
            change_type: ChangeType::Replace(scene_id),
            on_loaded: None,
            transition,
        });
    }

    /// Replace the current scene with a new one with transition and post-load callback.
    ///
    /// Like `replace_scene`, but executes a callback after the scene is loaded.
    /// This is useful for initializing the scene with specific data or state.
    ///
    /// # Parameters
    /// - `scene_id`: ID of the scene to load as replacement
    /// - `transition`: Optional transition effect to use for this scene change
    /// - `on_loaded`: Callback receiving mutable scene and context after loading
    pub fn replace_scene_with_callback(
        &mut self,
        scene_id: SceneId,
        transition: Option<Box<dyn SceneTransition>>,
        on_loaded: impl FnOnce(&mut dyn Scene, &mut SceneContext) + Send + 'static,
    ) {
        // Warn if overwriting a pending change (likely a bug)
        if self.pending_change.is_some() {
            log::warn!(
                target: "scene",
                "Overwriting pending scene change with replace to {:?} - this may cause issues",
                scene_id
            );
        }
        // Store the pending scene change with callback
        self.pending_change = Some(PendingSceneChange {
            change_type: ChangeType::Replace(scene_id),
            on_loaded: Some(Box::new(on_loaded)),
            transition,
        });
    }

    /// Process pending scene changes when transition-out completes.
    ///
    /// This is called automatically by `SceneManager::process()`. It:
    /// 1. Executes the scene change (push/pop/replace)
    /// 2. Runs any post-load callback
    /// 3. Starts the transition-in effect
    ///
    /// The transition-out effect should already be complete before this is called.
    pub fn process_pending_change(
        &mut self,
        ctx: &mut SceneContext,
        transition: &mut dyn SceneTransition,
    ) {
        // Only process if we have a pending change (transition-out should be done)
        if self.pending_change.is_none() {
            return;
        }

        let change = self.pending_change.take().unwrap();

        // Execute the scene change operation
        match change.change_type {
            ChangeType::Push(scene_id) => {
                self.pop_scene(ctx, true);
                self.push_scene(ctx, scene_id);
            }
            ChangeType::Pop => {
                self.pop_and_show_scene(ctx);
            }
            ChangeType::Replace(scene_id) => {
                self.pop_scene(ctx, false);
                self.push_scene(ctx, scene_id);
            }
        }

        // Execute callback after scene is loaded and active
        if let (Some(on_loaded), Some(scene)) = (change.on_loaded, &mut self.active_scene) {
            on_loaded(&mut **scene, ctx);
        }

        // Start transition in (fade from opaque to transparent)
        transition.transition_in(None);
    }

    /// Push a scene immediately without transition.
    ///
    /// Use this for initial scene setup when no transition is desired.
    pub fn change_push_scene_immediate(&mut self, ctx: &mut SceneContext, scene_id: SceneId) {
        self.pop_scene(ctx, true);
        self.push_scene(ctx, scene_id);
    }

    /// Replace a scene immediately without transition.
    pub fn replace_scene_immediate(&mut self, ctx: &mut SceneContext, scene_id: SceneId) {
        self.pop_scene(ctx, false);
        self.push_scene(ctx, scene_id);
    }

    /// Pop the current scene and restore the previous from history.
    fn pop_and_show_scene(&mut self, ctx: &mut SceneContext) {
        // Get the previous scene ID BEFORE popping from history
        // history has: [..., previous_scene, current_scene]
        // We want to restore previous_scene after popping current_scene
        let previous_scene_id = self
            .history
            .get(self.history.len().saturating_sub(2))
            .copied()
            .expect("cannot pop the root scene - need at least 2 scenes in history");

        // Pop the current scene (this will remove it from history)
        self.pop_scene(ctx, false);

        // Try to get the cached previous scene, or reload it if not cached
        // Note: Scene might not be cached if:
        // - It was marked as non-cacheable (can_cache() = false)
        // - Multiple transitions caused cache eviction
        // - The scene was explicitly cleared from cache
        let (mut scene, was_cached) =
            if let Some(scene) = self.unloaded_scenes.remove(&previous_scene_id) {
                (scene, true)
            } else {
                // Scene wasn't cached - need to recreate it
                log::debug!(
                    target: "scene",
                    "Reloading uncached scene {:?} during pop",
                    previous_scene_id
                );
                (self.scene_loader.load(previous_scene_id), false)
            };

        // Show scene - pass !was_cached so ready() is called for reloaded scenes
        self.show_scene(ctx, &mut *scene, was_cached);

        // set active scene
        self.active_scene.replace(scene);
    }

    /// Pop (remove) the current scene from the stack.
    ///
    /// # Parameters
    /// - `keep_history`: If true, keeps current scene ID in history (for replace operations)
    fn pop_scene(&mut self, ctx: &mut SceneContext, keep_history: bool) {
        if let Some(mut scene) = self.active_scene.take() {
            self.hide_scene(ctx, &mut *scene);

            // add to unloaded scenes
            if scene.can_cache() {
                self.unloaded_scenes.insert(scene.id(), scene);
            } else {
                scene.gd().queue_free();
            }
            // modify scene history stack
            if !keep_history {
                self.history.pop();
            }
        };
    }

    /// Push a new scene onto the stack, loading it if necessary.
    fn push_scene(&mut self, ctx: &mut SceneContext, scene_id: SceneId) {
        // take scene from loaded, and instantiate if none
        let mut has_loaded = true;
        let mut scene = self.unloaded_scenes.remove(&scene_id).unwrap_or_else(|| {
            has_loaded = false;
            self.scene_loader.load(scene_id)
        });

        self.show_scene(ctx, &mut *scene, has_loaded);

        // set active scene
        self.active_scene.replace(scene);

        // update history
        self.history.push(scene_id);
    }

    /// Show a scene by adding it to the tree and calling lifecycle hooks.
    ///
    /// # Parameters
    /// - `has_loaded`: Whether `ready()` has been called (true for cached scenes)
    fn show_scene(&mut self, ctx: &mut SceneContext, scene: &mut dyn Scene, has_loaded: bool) {
        // add to scene tree
        self.scene_root.add_child(&scene.gd());
        if !has_loaded {
            scene.ready(ctx);
        }
        scene.enter(ctx);
    }

    /// Hide a scene by calling leave hook and removing from tree.
    fn hide_scene(&mut self, ctx: &mut SceneContext, scene: &mut dyn Scene) {
        // call lifecycle function
        scene.leave(ctx);
        // remove from scene tree
        self.scene_root.remove_child(&scene.gd());
        // NOTE - this scene will be leaked
    }
}
