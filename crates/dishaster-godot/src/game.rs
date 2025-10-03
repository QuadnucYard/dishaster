pub mod perf;

use dishaster_core::{models::LevelConfig, sim::Simulation};
use dishaster_godot_ui::*;
use dishrupt_core::prelude::*;
use dishrupt_godot::{display::*, input::listener::GodotInputEvent};
use dishrupt_godot_scene::SceneContext;
use godot::{
    classes::{Node, Node2D},
    obj::Gd,
};

use crate::{
    dbgviz::*,
    game::perf::PerfTracker,
    game_main::GAME_DATA,
    runner::{SnapshotFrame, SyncSimulationRunner},
};

pub struct Game {
    // sim_runner: SimulationRunner,
    sim_runner: SyncSimulationRunner,
    stage: Stage,
    display_ctx: DisplayContext2D,
    dbgviz: DbgViz,

    perf_tracker: PerfTracker,
}

impl Game {
    pub fn new(gd: Gd<Node>, level: LevelConfig) -> Self {
        let db = GAME_DATA.get().unwrap();

        let map_prefab = &db
            .canteens
            .get_by_id(&db.levels.get_by_id(&level.id).unwrap().canteen)
            .unwrap()
            .display
            .res;

        let mut sim = Simulation::new(db.clone());
        sim.start(level);
        let root_entity = sim.root_entity();

        let sim_runner = SyncSimulationRunner::new(sim, 30.0);

        let mut stage_root = gd.get_node_as::<Node2D>("%Stage");
        let map_scene = load_prefab_sync(map_prefab).instantiate().unwrap();
        stage_root.add_child(&map_scene);
        let origin = map_scene
            .get_node_as::<Node2D>("%Origin")
            .get_global_position();

        let mut stage = Stage::new();
        let mut display_root_node = gd.get_node_as::<Node2D>("%DisplayRoot");
        display_root_node.set_position(origin);
        display_root_node.set_z_index(20);
        stage.set_root(root_entity, GdNode2D::new(display_root_node));
        let display_ctx = DisplayContext2D {
            view_scale: Vec3::new(60.0, 50.0, 50.0),
        };
        stage.set_display_context(display_ctx.clone());

        let dbgviz = DbgViz::new(&stage_root, origin);

        Self {
            sim_runner,
            stage,
            display_ctx,
            dbgviz,

            perf_tracker: Default::default(),
        }
    }

    pub fn process(&mut self, delta: f64, ctx: &mut SceneContext) {
        self.perf_tracker.tick_frame();

        if let Some(SnapshotFrame { ticks, snapshot }) = self.sim_runner.tick(delta) {
            self.perf_tracker.tick_updates(ticks);

            self.stage.present(snapshot.display.iter());
            self.dbgviz.update(&snapshot, &self.display_ctx);

            let hud = ctx.gui.get_mut::<TimeStatsGui>();
            hud.update_time(snapshot.sim_tick, snapshot.sim_time_seconds);
        }

        // update perf stats
        self.perf_tracker.sample(delta);

        // update HUD
        let hud = ctx.gui.get_mut::<TimeStatsGui>();
        hud.update_perf(self.perf_tracker.last_fps, self.perf_tracker.last_ups);
    }

    pub fn process_input(&mut self, _event: GodotInputEvent) {
        // TODO: Add your actual input event handling here
        // You can call self.pause(), self.resume(), and self.is_paused() from your input handling code
        // Example: if space key pressed, toggle pause state
    }

    /*
    /// Pause the simulation.
    pub fn pause(&self) {
        self.sim_runner.pause();
    }

    /// Resume the simulation.
    pub fn resume(&self) {
        self.sim_runner.resume();
    }

    /// Check if simulation is currently paused.
    pub fn is_paused(&self) -> bool {
        self.sim_runner.is_paused()
    }
    */
}
