//! Godot presentation layer for opening animation

use dishaster_opening::{
    OpeningSimulationFeat, Simulation as OpeningSimulation, resources::OpeningConfig,
};
use dishrupt_core::{EntityId, prelude::*};
use dishrupt_godot::display::{DisplayContext2D, GdNode2D, Stage};
use dishrupt_runner::{ISimulation, SimulationRunner, SnapshotFrame, SyncSimulationRunner};
use godot::{
    classes::{Node, Node2D},
    prelude::*,
};

/// Presenter for opening animation that uses Stage for node management
pub struct Opening {
    sim_runner: Box<dyn SimulationRunner<OpeningSimulationFeat>>,
    stage: Stage,
}

impl Opening {
    /// Create a new opening presenter
    pub fn new(root: Gd<Node>) -> Self {
        // Initialize simulation
        let sim = Box::new(OpeningSimulation::new(
            OpeningConfig::default(),
            godot::global::randi() as u64,
        ));
        let root_entity = sim.root_entity();
        let sim_runner = Box::new(SyncSimulationRunner::new(sim, 60.0));

        let stage = Self::setup_stage(&root, root_entity);

        Self { sim_runner, stage }
    }

    fn setup_stage(root: &Gd<Node>, root_entity: EntityId) -> Stage {
        // Opening world is 20x12 meters. With viewport 1920x1080,
        // we want the 20m width to fill most of the screen. 1920/20 = 96 pixels per meter.
        let display_ctx = DisplayContext2D {
            view_scale: vec3(100., -100., 100.),
        };
        let mut stage = Stage::new(display_ctx);
        let mut display_root_node = root.get_node_as::<Node2D>("%DisplayRoot");
        let origin = root.get_node_as::<Node2D>("%Origin").get_global_position();
        display_root_node.set_position(origin);
        stage.set_root(root_entity, GdNode2D::new(display_root_node));
        stage
    }

    /// Process the opening presenter
    pub fn process(&mut self, _delta: f64) {
        if let Some(SnapshotFrame { snapshot, .. }) = self.sim_runner.tick(1.0 / 60.0) {
            self.stage.present(snapshot.display.iter());
        }
    }
}
