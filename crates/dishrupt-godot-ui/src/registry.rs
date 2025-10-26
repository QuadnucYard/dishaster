use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

use as_any::AsAny;

use crate::UITree;

pub trait Gui: UITree {
    fn start(&mut self, _commands: GuiCommands) {}

    fn process(&mut self, _commands: GuiCommands, _delta: f64) {}
}

#[derive(Default)]
pub struct GuiRegistry {
    // We use a Vec here to ensure stable addresses for the Gui trait objects.
    guis: Vec<(TypeId, Box<dyn Gui>)>,
}

impl GuiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<G: Gui + 'static>(&mut self, gui: G) {
        self.guis.push((TypeId::of::<G>(), Box::new(gui)));
    }

    pub fn get<G: Gui + 'static>(&self) -> &G {
        let id = TypeId::of::<G>();
        self.guis
            .iter()
            .find(|(ty, _)| id == *ty)
            .map(|(_, g)| unsafe { &*Box::as_ptr(g).cast::<G>() })
            .unwrap()
    }

    pub fn get_mut<G: Gui + 'static>(&mut self) -> &mut G {
        let id = TypeId::of::<G>();
        self.guis
            .iter_mut()
            .find(|(ty, _)| id == *ty)
            .map(|(_, g)| unsafe { &mut *Box::as_mut_ptr(g).cast::<G>() })
            .unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Gui>> {
        self.guis.iter().map(|(_, g)| g)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Gui>> {
        self.guis.iter_mut().map(|(_, g)| g)
    }

    pub fn show<G: Gui + 'static>(&mut self) {
        self.get_mut::<G>().show();
    }

    pub fn hide<G: Gui + 'static>(&mut self) {
        self.get_mut::<G>().hide();
    }

    pub fn hide_all(&mut self) {
        for gui in self.iter_mut() {
            gui.hide();
        }
    }
}

type GuiCmd = Box<dyn FnOnce(&mut GuiRegistry) + Send + 'static>;

#[derive(Default, Clone)]
pub struct GuiCommands(Arc<Mutex<GuiCommandsRepr>>);

#[derive(Default)]
struct GuiCommandsRepr {
    pub cmds: Vec<GuiCmd>,
    pub reqs: Vec<Box<dyn GuiRequest>>,
}

impl GuiCommands {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_cmd<F>(&self, f: F)
    where
        F: FnOnce(&mut GuiRegistry) + Send + 'static,
    {
        self.0.lock().unwrap().cmds.push(Box::new(f));
    }

    pub fn push_req<R>(&self, req: R)
    where
        R: GuiRequest + 'static,
    {
        self.0.lock().unwrap().reqs.push(Box::new(req));
    }

    pub fn run_cmds(&mut self, gui: &mut GuiRegistry) {
        // Drain the commands to avoid holding the lock while executing them
        let cmds = self.0.lock().unwrap().cmds.drain(..).collect::<Vec<_>>();
        for cmd in cmds {
            cmd(gui);
        }
    }

    pub fn run_reqs(&mut self, f: impl FnMut(Box<dyn GuiRequest>)) {
        self.take_reqs().into_iter().for_each(f);
    }

    pub fn take_reqs(&mut self) -> Vec<Box<dyn GuiRequest>> {
        self.0.lock().unwrap().reqs.drain(..).collect::<Vec<_>>()
    }
}

/// Type-erased request to the GUI system.
pub trait GuiRequest: Send + AsAny {}
