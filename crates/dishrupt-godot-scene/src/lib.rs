mod manager;
mod proc;

use as_any::AsAny;
use dishrupt_godot::{audio::AudioManager, input::listener::GodotInputEvent};
use dishrupt_godot_ui::{GuiCommands, GuiRegistry};
use godot::{classes::Node, obj::Gd};
pub use manager::*;
pub use proc::*;

pub type SceneId = &'static str;

pub struct SceneContext<'a> {
    pub gui: &'a mut GuiRegistry,
    pub gui_cmds: GuiCommands,
    pub audio: &'a mut AudioManager,
    pub proc: Option<Box<dyn SceneProcedure>>,
}

impl SceneContext<'_> {
    pub fn schedule(&mut self, f: impl SceneProcedure + 'static) {
        self.proc.replace(Box::new(f));
    }
}

pub struct SceneProcedureContext<'a> {
    pub base: &'a mut SceneContext<'a>,
    pub scene_stack: &'a mut SceneStack,
}

pub enum SceneTransition {
    Push(SceneId),
    Replace(SceneId),
    Pop,
}

#[allow(unused)]
pub trait Scene: AsAny {
    fn id(&self) -> SceneId;

    fn gd(&self) -> Gd<Node>;

    /// Whether the scene can be cached when unloaded
    fn can_cache(&self) -> bool {
        true
    }

    fn ready(&mut self, ctx: &mut SceneContext) {}

    fn enter(&mut self, ctx: &mut SceneContext) {}

    fn leave(&mut self, ctx: &mut SceneContext) {
        ctx.gui.hide_all();
    }

    fn process(&mut self, ctx: &mut SceneContext, delta: f64) {}

    fn physics_process(&mut self, ctx: &mut SceneContext, delta: f64) {}

    fn input(&mut self, ctx: &mut SceneContext, event: GodotInputEvent) {}
}
