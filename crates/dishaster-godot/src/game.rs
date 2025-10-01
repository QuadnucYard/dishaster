use dishaster_core::{models::LevelConfig, sim::Simulation};
use dishaster_godot_ui::*;
use dishrupt_core::prelude::*;
use dishrupt_godot::{display::*, input::listener::GodotInputEvent};
use dishrupt_godot_scene::SceneContext;
use godot::{
    classes::{Node, Node2D},
    obj::Gd,
};

use crate::{dbgviz::*, game_main::GAME_DATA, runner::SyncSimulationRunner};

const METRIC_SAMPLE_INTERVAL: f64 = 0.5;

pub struct Game {
    // sim_runner: SimulationRunner,
    sim_runner: SyncSimulationRunner,
    stage: Stage,
    display_ctx: DisplayContext2D,
    dbgviz: DbgViz,

    time_stats: TimeStats,
}

#[derive(Debug, Default)]
pub struct TimeStats {
    pub frame_accum: f64,
    pub frame_count: u32,
    pub fps_estimate: f64,
    pub update_accum: f64,
    pub update_count: u32,
    pub ups_estimate: f64,
    pub last_sim_time: f64,
    pub last_sim_tick: u64,
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

        let sim_runner = SyncSimulationRunner::new(sim);

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

            time_stats: Default::default(),
        }
    }

    pub fn process(&mut self, delta: f64, ctx: &mut SceneContext) {
        let time_stats = &mut self.time_stats;
        time_stats.frame_accum += delta;
        time_stats.frame_count += 1;

        if time_stats.frame_accum >= METRIC_SAMPLE_INTERVAL {
            time_stats.fps_estimate = time_stats.frame_count as f64 / time_stats.frame_accum;
            time_stats.frame_accum = 0.0;
            time_stats.frame_count = 0;
        }

        time_stats.update_accum += delta;

        if let Some(snapshot) = self.sim_runner.tick(delta) {
            self.stage.present(snapshot.display.iter());
            self.dbgviz.update(&snapshot, &self.display_ctx);

            time_stats.update_count += 1; // TODO: count actual sim steps
            time_stats.last_sim_time = snapshot.sim_time_seconds;
            time_stats.last_sim_tick = snapshot.sim_tick;
        }

        if time_stats.update_accum >= METRIC_SAMPLE_INTERVAL {
            time_stats.ups_estimate = if time_stats.update_count > 0 {
                time_stats.update_count as f64 / time_stats.update_accum
            } else {
                f64::NAN
            };
            time_stats.update_accum = 0.0;
            time_stats.update_count = 0;
        }

        let hud = ctx.gui.get_mut::<TimeStatsGui>();
        hud.update(&TimeStatsD {
            fps_estimate: time_stats.fps_estimate,
            ups_estimate: time_stats.ups_estimate,
            last_sim_time: time_stats.last_sim_time,
            last_sim_tick: time_stats.last_sim_tick,
        });
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
