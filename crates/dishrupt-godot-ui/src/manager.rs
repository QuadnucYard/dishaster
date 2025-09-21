use godot::{classes::Node, obj::Gd};

use crate::{GuiCommands, GuiRegistry, UIRoot};

pub struct GuiManager {
    pub root: UIRoot,
    pub registry: GuiRegistry,
    pub cmds: GuiCommands,
}

impl GuiManager {
    pub fn new(root_gd: Gd<Node>) -> Self {
        Self {
            root: UIRoot::new(root_gd),
            registry: GuiRegistry::new(),
            cmds: GuiCommands::default(),
        }
    }

    pub fn ready(&mut self) {
        // We ensure that all ui tree are added to the root.
        for gui in self.registry.iter_mut() {
            gui.start(self.cmds.clone());
            gui.ready();
            gui.set_active(false);
            self.root.add_gui(&**gui);
        }
        self.cmds.run_cmds(&mut self.registry);
    }

    pub fn process(&mut self) {
        for gui in self.registry.iter_mut() {
            if gui.is_active() {
                gui.process();
            }
        }
    }
}
