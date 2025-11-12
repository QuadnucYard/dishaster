//! Godot presentation layer for opening animation

mod present;

use dishaster_opening::{
    OpeningSimulationFeat, Simulation as OpeningSimulation,
    protocol::{ItemType, ObjectSnapshot, SimEvent},
    resources::OpeningConfig,
};
use dishrupt_asset::AssetCatalog;
use dishrupt_core::{EntityId, prelude::*};
use dishrupt_godot_display::{DisplayContext2D, GdNode2D, Stage};
use dishrupt_runner::{ISimulation, SimulationRunner, SnapshotFrame, SyncSimulationRunner};
use godot::{
    classes::{Node, Node2D},
    prelude::*,
};
use present::{DishPresenter, EmojiPresenter, TextPresenter};
use rustc_hash::FxHashMap;

/// Presenter for opening animation that uses Stage for node management
pub struct Opening {
    sim_runner: Box<dyn SimulationRunner<OpeningSimulationFeat>>,
    stage: Stage,
    presenters: Presenters,
}

#[derive(Default)]
/// Container for all presenters
struct Presenters {
    foods: FxHashMap<EntityId, DishPresenter>,
    faces: FxHashMap<EntityId, EmojiPresenter>,
    texts: FxHashMap<EntityId, TextPresenter>,
}

impl Opening {
    /// Create a new opening presenter
    pub fn new(root: Gd<Node>, asset_catalog: AssetCatalog) -> Self {
        // Initialize simulation
        let sim = Box::new(OpeningSimulation::new(
            OpeningConfig::default(),
            godot::global::randi() as u64,
        ));
        let root_entity = sim.root_entity();
        let sim_runner = Box::new(SyncSimulationRunner::new(sim, 60.0));

        let stage = Self::setup_stage(&root, root_entity, asset_catalog);

        Self {
            sim_runner,
            stage,
            presenters: Default::default(),
        }
    }

    fn setup_stage(root: &Gd<Node>, root_entity: EntityId, asset_catalog: AssetCatalog) -> Stage {
        // Opening world is 20x12 meters. With viewport 1920x1080,
        // we want the 20m width to fill most of the screen. 1920/20 = 96 pixels per meter.
        let display_ctx = DisplayContext2D {
            view_scale: vec3(100., -100., 100.),
        };
        let mut stage = Stage::new(display_ctx, asset_catalog);
        let mut display_root_node = root.get_node_as::<Node2D>("%DisplayRoot");
        let origin = root.get_node_as::<Node2D>("%Origin").get_global_position();
        display_root_node.set_position(origin);
        stage.set_root(root_entity, GdNode2D::new(display_root_node));
        stage
    }

    /// Process the opening presenter
    pub fn process(&mut self, delta: f64) {
        if let Some(SnapshotFrame {
            snapshot, events, ..
        }) = self.sim_runner.tick(delta)
        {
            // Stage handles node instantiation and positioning
            self.stage.present(snapshot.display.iter());

            // Process events to create/destroy presenters
            self.process_events(events);

            // Update presenters with dynamic visual effects
            self.update_presenters(&snapshot.objects);
        }
    }

    /// Process events from simulation
    fn process_events(&mut self, events: Vec<SimEvent>) {
        for event in events {
            match event {
                SimEvent::FoodSpawned {
                    entity,
                    variant,
                    color,
                } => {
                    if let Some(node) = self.stage.get_godot_node(entity).cloned() {
                        let presenter = DishPresenter::new(node, variant, color);
                        self.presenters.foods.insert(entity, presenter);
                    }
                }
                SimEvent::FaceSpawned { entity, variant } => {
                    if let Some(node) = self.stage.get_godot_node(entity).cloned() {
                        let presenter = EmojiPresenter::new(node, variant);
                        self.presenters.faces.insert(entity, presenter);
                    }
                }
                SimEvent::TextSpawned { entity, content } => {
                    if let Some(node) = self.stage.get_godot_node(entity).cloned() {
                        let presenter = TextPresenter::new(node, content);
                        self.presenters.texts.insert(entity, presenter);
                    }
                }
                SimEvent::ObjectDespawned(entity) => {
                    self.presenters.foods.remove(&entity);
                    self.presenters.faces.remove(&entity);
                    self.presenters.texts.remove(&entity);
                }
            }
        }
    }

    /// Update presenters with snapshot data
    fn update_presenters(&mut self, presentation: &[ObjectSnapshot]) {
        for snap in presentation {
            match snap.item_type {
                ItemType::Food => {
                    if let Some(presenter) = self.presenters.foods.get_mut(&snap.entity) {
                        presenter.update(snap.alpha);
                    }
                }
                ItemType::Face => {
                    if let Some(presenter) = self.presenters.faces.get_mut(&snap.entity) {
                        presenter.update(snap.alpha);
                    }
                }
                ItemType::Text => {
                    if let Some(presenter) = self.presenters.texts.get_mut(&snap.entity) {
                        presenter.update(snap.alpha, snap.wave_phase, snap.color);
                    }
                }
            }
        }
    }
}
