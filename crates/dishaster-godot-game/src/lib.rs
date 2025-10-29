//! Dishaster Godot Game Module

mod ctrl;
mod dbgviz;
mod handle_request;
mod input;
pub mod perf;
mod present;
pub mod user_store;

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use dishaster_godot_ui::*;
use dishaster_interface::*;
use dishaster_models::{GameModelRegistry, LevelConfig};
use dishaster_persistence::ProgressService;
use dishaster_runner::{SimulationRunner, SnapshotFrame, SyncSimulationRunner};
use dishrupt_core::{EntityId, prelude::*};
use dishrupt_godot::display::*;
use dishrupt_godot_scene::SceneContext;
use dishrupt_l10n::tr;
use godot::{
    classes::{Node, Node2D},
    prelude::*,
};
use rustc_hash::FxHashMap;

use self::{ctrl::*, perf::PerfTracker};
use crate::{dbgviz::*, user_store::GodotUserStorage};

pub static GAME_DATA: OnceLock<Arc<GameModelRegistry>> = OnceLock::new();
pub static PROGRESS_SERVICE: OnceLock<Mutex<ProgressService<GodotUserStorage>>> = OnceLock::new();

pub fn progress_service() -> MutexGuard<'static, ProgressService<GodotUserStorage>> {
    PROGRESS_SERVICE
        .get()
        .expect("progress service not initialized")
        .lock()
        .expect("progress service poisoned")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DayPhase {
    Preparation,
    Running,
    Settlement,
}

#[derive(Debug, Default, Clone, Copy)]
struct DayTelemetry {
    #[allow(unused)]
    seed: u64,
    day: u32,
    tick: u32,
    seconds: f64,
}

pub struct Game {
    root: Gd<Node>,

    sim_runner: Box<dyn SimulationRunner>,
    stage: Stage,
    stage_origin: Vector2,
    dbgviz: DbgViz,

    perf_tracker: PerfTracker,
    dc: DisplayControllers,

    phase: DayPhase,
    telemetry: DayTelemetry,
    debug_enabled: bool,
    suspended_sim_speed: Option<f64>,
}

#[derive(Default)]
struct DisplayControllers {
    agents: FxHashMap<EntityId, AgentController>,
    dishes: FxHashMap<EntityId, DishController>,
}

impl Game {
    pub fn new(
        gd: Gd<Node>,
        level: LevelConfig,
        sim_creator: impl FnOnce(Arc<GameModelRegistry>) -> Box<dyn ISimulation>,
    ) -> Self {
        let db = GAME_DATA.get().expect("game data not initialized");

        let map_prefab = &db
            .canteens
            .get_by_id(&db.levels.get_by_id(&level.id).unwrap().canteen)
            .unwrap()
            .display
            .res;

        // Set up the map scene
        let mut stage_root = gd.get_node_as::<Node2D>("%Stage");
        let map_scene = load_prefab_sync(map_prefab).instantiate().unwrap();
        stage_root.add_child(&map_scene);
        let origin = map_scene
            .get_node_as::<Node2D>("%Origin")
            .get_global_position();

        // Position the display root at the map origin
        let mut display_root_node = gd.get_node_as::<Node2D>("%DisplayRoot");
        display_root_node.set_position(origin);
        display_root_node.set_z_index(20);
        let display_root = GdNode2D::new(display_root_node);
        let display_ctx = DisplayContext2D {
            view_scale: Vec3::new(60.0, 50.0, 50.0),
        };

        let telemetry = DayTelemetry {
            seed: level.seed,
            day: level.day,
            tick: 0,
            seconds: 0.0,
        };

        // Initialize simulation
        let mut sim = sim_creator(db.clone());
        sim.start(level);
        let root_entity = sim.root_entity();
        let default_tps = 60.0;
        let sim_runner = SyncSimulationRunner::new(sim, default_tps);

        // Set up stage
        let mut stage = Stage::new(display_ctx);
        stage.set_root(root_entity, display_root.clone());

        // Set up debug visualization
        let dbgviz = DbgViz::new(&stage_root, origin);

        Self {
            root: gd,
            sim_runner: Box::new(sim_runner),
            stage,
            stage_origin: origin,
            dbgviz,
            perf_tracker: Default::default(),
            dc: Default::default(),
            phase: DayPhase::Preparation,
            telemetry,
            debug_enabled: false,
            suspended_sim_speed: None,
        }
    }

    pub fn process(&mut self, delta: f64, ctx: &mut SceneContext) {
        self.perf_tracker.tick_frame();

        if let Some(SnapshotFrame {
            ticks,
            snapshot,
            events,
            responses,
        }) = self.sim_runner.tick(delta)
        {
            self.perf_tracker.tick_updates(ticks);

            self.stage.present(snapshot.display.iter());
            self.dbgviz
                .update(&snapshot.debug, self.stage.display_context());
            self.update_other_debug(&snapshot.debug);

            self.process_events(ctx, events);
            self.process_query_responses(ctx, responses);

            self.telemetry.tick = snapshot.sim_tick;
            self.telemetry.seconds = snapshot.sim_time_seconds;
        }

        self.process_display(delta);
        self.perf_tracker.sample(delta);
        self.update_hud(ctx);
    }

    pub fn send_sim_command(&mut self, command: SimCommand) {
        self.sim_runner.send_command(command);
    }

    pub fn send_sim_query(&mut self, query: SimQuery) {
        self.sim_runner.send_query(query);
    }

    /// Called just after construction
    pub fn start_day(&mut self, ctx: &mut SceneContext) {
        ctx.gui.get_mut::<GamingLayout>().apply_state(&DayHudState {
            day_label: tr!("day-display.label", "day" = self.telemetry.day),
            phase_label: tr!("phase-preparation.label"),
            details: "Review canteen status then press Start Day to begin.".into(),
            show_start: true,
            enable_start: true,
            show_dev: true,
            enable_dev: false,
        });

        ctx.gui
            .get_mut::<TimeStatsGui>()
            .set_tps_display(self.sim_runner.tps() as f32);

        ctx.gui.get_mut::<DishPricePopup>().enabled = true;

        self.send_sim_query(SimQuery::Distances);
    }

    pub fn begin_run(&mut self, ctx: &mut SceneContext) {
        if self.phase != DayPhase::Preparation {
            return;
        }
        self.phase = DayPhase::Running;

        ctx.gui.get_mut::<DishPricePopup>().enabled = false;

        self.send_sim_command(SimCommand::StartRun);
    }

    pub fn force_finish_day(&mut self, ctx: &mut SceneContext) {
        if self.phase == DayPhase::Running {
            self.finish_day(ctx, true);
        }
    }

    pub fn finish_day(&mut self, ctx: &mut SceneContext, forced: bool) {
        if self.phase != DayPhase::Running {
            return;
        }
        self.phase = DayPhase::Settlement;

        if forced {
            self.send_sim_command(SimCommand::EndRun);
            self.dc.agents.clear();
        }

        progress_service()
            .complete_day()
            .expect("failed to complete day");

        ctx.gui.hide::<GamingLayout>();
        ctx.gui.show::<SettlementGui>();
    }

    fn update_hud(&mut self, ctx: &mut SceneContext) {
        if self.phase == DayPhase::Running {
            let layout = ctx.gui.get_mut::<GamingLayout>();
            let state = DayHudState {
                day_label: tr!("day-display.label", "day" = self.telemetry.day),
                phase_label: tr!("phase-running.label"),
                details: "Service running.".to_string(),
                show_start: false,
                enable_start: false,
                show_dev: true,
                enable_dev: true,
            };
            layout.apply_state(&state);
        }

        {
            let stats = ctx.gui.get_mut::<TimeStatsGui>();
            stats.update_time(self.telemetry.tick, self.telemetry.seconds);
            stats.update_perf(self.perf_tracker.last_fps, self.perf_tracker.last_ups);
        }
    }
}
