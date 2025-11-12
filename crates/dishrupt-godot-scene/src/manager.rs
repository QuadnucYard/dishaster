use std::collections::HashMap;

use godot::{classes::Node, obj::Gd};

use crate::{
    Scene, SceneContext, SceneId, SceneProcedureContext,
    proc::{SceneProcedure, SceneProcedurePoll},
};

pub trait SceneLoader {
    fn load(&self, id: SceneId) -> Box<dyn Scene>;
}

pub struct SceneManager {
    stack: SceneStack,
    proc: Option<Box<dyn SceneProcedure>>,
}

impl SceneManager {
    pub fn new(scene_root: Gd<Node>, scene_loader: impl SceneLoader + 'static) -> Self {
        Self {
            stack: SceneStack::new(scene_root, scene_loader),
            proc: Default::default(),
        }
    }

    pub fn active_scene(&self) -> &Option<Box<dyn Scene>> {
        &self.stack.active_scene
    }

    pub fn active_scene_mut(&mut self) -> &mut Option<Box<dyn Scene>> {
        &mut self.stack.active_scene
    }

    pub fn inspect_active_scene(&self, f: impl FnOnce(&dyn Scene)) {
        self.stack.inspect_active_scene(f);
    }

    pub fn inspect_active_scene_mut(&mut self, f: impl FnOnce(&mut dyn Scene)) {
        self.stack.inspect_active_scene_mut(f);
    }

    pub fn schedule(&mut self, f: impl SceneProcedure + 'static) {
        self.proc.replace(Box::new(f));
    }

    pub fn schedule1(&mut self, proc: Box<dyn SceneProcedure>) {
        self.proc.replace(proc);
    }

    pub fn process<'a>(&'a mut self, ctx: &'a mut SceneContext<'a>) {
        if let Some(mut proc) = self.proc.take() {
            log::debug!("Processing scene procedure");
            let ctx = &mut SceneProcedureContext {
                base: ctx,
                scene_stack: &mut self.stack,
            };
            let r = proc.process(ctx);
            if matches!(r, SceneProcedurePoll::Pending) {
                self.proc.replace(proc);
            }
        }
    }
}

pub struct SceneStack {
    active_scene: Option<Box<dyn Scene>>,
    unloaded_scenes: HashMap<SceneId, Box<dyn Scene>>,
    history: Vec<SceneId>,

    scene_root: Gd<Node>,
    scene_loader: Box<dyn SceneLoader>,
}

impl SceneStack {
    pub fn new(scene_root: Gd<Node>, scene_loader: impl SceneLoader + 'static) -> Self {
        Self {
            scene_root,
            active_scene: Default::default(),
            unloaded_scenes: Default::default(),
            history: Default::default(),

            scene_loader: Box::new(scene_loader),
        }
    }

    pub fn inspect_active_scene(&self, f: impl FnOnce(&dyn Scene)) {
        self.active_scene.as_ref().inspect(|&scene| f(&**scene));
    }

    pub fn inspect_active_scene_mut(&mut self, f: impl FnOnce(&mut dyn Scene)) {
        if let Some(scene) = self.active_scene.as_mut() {
            f(&mut **scene);
        }
    }

    /// Change current scene to a new one.
    pub fn change_push_scene(&mut self, ctx: &mut SceneContext, scene_id: SceneId) {
        self.pop_scene(ctx, true);
        self.push_scene(ctx, scene_id);
    }

    /// Change current scene to the last one.
    pub fn change_pop_scene(&mut self, ctx: &mut SceneContext) {
        self.pop_and_show_scene(ctx);
    }

    /// Replace the current scene with a scene typed `T`.
    ///
    /// It will replace the scene history.
    pub fn replace_scene(&mut self, ctx: &mut SceneContext, scene_id: SceneId) {
        self.pop_scene(ctx, false);
        self.push_scene(ctx, scene_id);
    }

    fn pop_and_show_scene(&mut self, ctx: &mut SceneContext) {
        self.pop_scene(ctx, false);

        // get scene
        let mut scene = self
            .unloaded_scenes
            .remove(
                self.history
                    .last()
                    .expect("the current scene is not the root"),
            )
            .expect("the wanted scene has been loaded");

        self.show_scene(ctx, &mut *scene, true);

        // set active scene
        self.active_scene.replace(scene);
    }

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

    fn show_scene(&mut self, ctx: &mut SceneContext, scene: &mut dyn Scene, has_loaded: bool) {
        // add to scene tree
        self.scene_root.add_child(&scene.gd());
        if !has_loaded {
            scene.ready(ctx);
        }
        scene.enter(ctx);
    }

    fn hide_scene(&mut self, ctx: &mut SceneContext, scene: &mut dyn Scene) {
        // call lifecycle function
        scene.leave(ctx);
        // remove from scene tree
        self.scene_root.remove_child(&scene.gd());
        // NOTE - this scene will be leaked
    }
}
