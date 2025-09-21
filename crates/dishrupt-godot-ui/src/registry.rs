use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use as_any::AsAny;

use crate::UITree;

pub trait Gui: UITree {
    fn start(&mut self, _commands: GuiCommands) {}
}

#[derive(Default)]
pub struct GuiRegistry {
    guis: HashMap<TypeId, Box<dyn Gui>>,
}

impl GuiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<G: Gui + 'static>(&mut self, gui: G) {
        self.guis.insert(TypeId::of::<G>(), Box::new(gui));
    }

    pub fn get<G: Gui + 'static>(&self) -> &G {
        self.guis
            .get(&TypeId::of::<G>())
            .map(|g| unsafe { &*Box::as_ptr(g).cast::<G>() })
            .unwrap()
    }

    pub fn get_mut<G: Gui + 'static>(&mut self) -> &mut G {
        self.guis
            .get_mut(&TypeId::of::<G>())
            .map(|g| unsafe { &mut *Box::as_mut_ptr(g).cast::<G>() })
            .unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Gui>> {
        self.guis.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Gui>> {
        self.guis.values_mut()
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
        let mut guard = self.0.lock().unwrap();
        for cmd in guard.cmds.drain(..) {
            cmd(gui);
        }
    }

    pub fn run_reqs(&mut self, f: impl FnMut(Box<dyn GuiRequest>)) {
        let mut guard = self.0.lock().unwrap();
        guard.reqs.drain(..).for_each(f);
    }
}

/// Type-erased request to the GUI system.
pub trait GuiRequest: Send + AsAny {}
