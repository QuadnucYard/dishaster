//! Dishaster Godot Game Module

mod dbgviz;
mod handle_request;
mod hint;
mod input;
pub mod perf;
mod present;
pub mod user_store;

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use dishaster_interface::*;
use dishaster_models::{GameModelRegistry, LevelSetupState};
use dishaster_persistence::PlayerService;
use dishaster_ui_protocol::{StatsView, UiCommand};
use dishaster_views::DayHudState;
use dishrupt_core::prelude::*;
use dishrupt_godot::{NodeExt, display::*};
use dishrupt_l10n::tr;
use dishrupt_runner::{ISimulation, SimulationRunner, SnapshotFrame, SyncSimulationRunner};
use godot::{
    classes::{Node, Node2D},
    prelude::*,
};
use rustc_hash::FxHashMap;

use self::{perf::PerfTracker, present::*};
use crate::{dbgviz::*, hint::HintTracker, user_store::GodotUserStorage};

pub static GAME_DATA: OnceLock<Arc<GameModelRegistry>> = OnceLock::new();
pub static PROGRESS_SERVICE: OnceLock<Mutex<PlayerService<GodotUserStorage>>> = OnceLock::new();

pub fn progress_service() -> MutexGuard<'static, PlayerService<GodotUserStorage>> {
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
    /// Current number of live diners in the canteen
    live_diners: usize,
    /// Total number of diners that have visited this day
    total_visits: usize,
}

pub struct Game {
    root: Gd<Node>,

    sim_runner: Box<dyn SimulationRunner<CoreSimulationFeat>>,
    stage: Stage,
    stage_origin: Vector2,
    dbgviz: DbgViz,

    perf_tracker: PerfTracker,
    pres: Presenters,
    hint_tracker: HintTracker,

    phase: DayPhase,
    telemetry: DayTelemetry,
    debug_enabled: bool,
    suspended_sim_speed: Option<f64>,

    /// Queue of UI commands to be processed by the scene layer.
    ui_commands: Vec<UiCommand>,
}

#[derive(Default)]
struct Presenters {
    agents: FxHashMap<EntityId, AgentPresenter>,
    dishes: FxHashMap<EntityId, DishPresenter>,
    dispensers: FxHashMap<EntityId, DispenserPresenter>,
}

impl Game {
    /// Create a new game instance with the given root node and level configuration.
    ///
    /// NOTE: the creator should START the simulation after construction
    pub fn new(
        gd: Gd<Node>,
        level: LevelSetupState,
        sim_creator: impl FnOnce(
            Arc<GameModelRegistry>,
            LevelSetupState,
        ) -> Box<dyn ISimulation<CoreSimulationFeat>>,
    ) -> Self {
        let db = GAME_DATA.get().expect("game data not initialized");

        let telemetry = DayTelemetry {
            seed: level.seed,
            day: level.day,
            ..Default::default()
        };

        let levle_config = db
            .levels
            .get_by_id(&level.level_id)
            .expect("failed to get level");
        let map_prefab = &db
            .canteens
            .get_by_id(&levle_config.canteen)
            .expect("failed to get canteen")
            .display
            .res;

        // Initialize simulation
        let sim = sim_creator(db.clone(), level);
        let root_entity = sim.root_entity();
        let default_tps = 60.0;
        let sim_runner = SyncSimulationRunner::new(sim, default_tps);

        // Set up the map scene
        let mut stage_root = gd.get_node_as::<Node2D>("%Stage");
        let map_scene = load_prefab_sync(map_prefab)
            .instantiate()
            .expect("failed to instantiate map prefab");
        stage_root.add_child(&map_scene);
        let origin = map_scene
            .get_node_as::<Node2D>("%Origin")
            .get_global_position();

        // Position the display root at the map origin
        let mut display_root_node = gd.get_node_as::<Node2D>("%DisplayRoot");
        display_root_node.set_position(origin);
        display_root_node.set_z_index(20);

        // Set up stage
        let display_ctx = DisplayContext2D {
            view_scale: Vec3::new(60.0, 50.0, 50.0),
        };
        let mut stage = Stage::new(display_ctx);
        stage.set_root(root_entity, GdNode2D::new(display_root_node));

        // Set up debug visualization
        let dbgviz = {
            let mut debug_root = stage_root.get_or_add_node_as::<Node2D>("Debug");
            debug_root.set_position(origin);
            DbgViz::new(debug_root)
        };

        Self {
            root: gd,
            sim_runner: Box::new(sim_runner),
            stage,
            stage_origin: origin,
            dbgviz,

            perf_tracker: Default::default(),
            pres: Default::default(),
            hint_tracker: HintTracker::new(
                progress_service().profile().progress.seen_hints.clone(),
            ),

            phase: DayPhase::Preparation,
            telemetry,
            debug_enabled: false,
            suspended_sim_speed: None,
            ui_commands: Vec::new(),
        }
    }

    pub fn process(&mut self, delta: f64) {
        self.perf_tracker.tick_frame();

        if let Some(SnapshotFrame {
            delta_ticks,
            snapshot,
            events,
            responses,
            ..
        }) = self.sim_runner.tick(delta)
        {
            self.perf_tracker.tick_updates(delta_ticks);

            self.stage.present(snapshot.display.iter());
            self.dbgviz
                .update(&snapshot.debug, self.stage.display_context());
            self.update_other_debug(&snapshot.debug);

            self.process_events(events);
            self.process_query_responses(responses);

            self.telemetry.tick = snapshot.stats.tick;
            self.telemetry.seconds = snapshot.stats.time_seconds;
            self.telemetry.live_diners = snapshot.stats.live_diners;
            self.telemetry.total_visits = snapshot.stats.total_visits;
        }

        self.process_display(delta);
        self.perf_tracker.sample(delta);
        self.update_hud();
    }

    /// Poll and drain UI commands that need to be processed by the scene layer.
    pub fn poll_ui_commands(&mut self) -> Vec<UiCommand> {
        std::mem::take(&mut self.ui_commands)
    }

    pub fn send_sim_command(&mut self, command: SimCommand) {
        self.sim_runner.send_command(command);
    }

    pub fn send_sim_query(&mut self, query: SimQuery) {
        self.sim_runner.send_query(query);
    }

    /// Called just after construction
    pub fn start_day(&mut self) {
        self.ui_commands.push(UiCommand::UpdateDayHud(
            DayHudState {
                day_label: tr!("day-display.label", "day" = self.telemetry.day),
                phase_label: tr!("phase-preparation.label"),
                details: "Review canteen status then press Start Day to begin.".into(),
                show_start: true,
                enable_start: true,
                show_dev: true,
                enable_dev: false,
            }
            .into(),
        ));
        self.ui_commands
            .push(UiCommand::UpdateTpsDisplay(self.sim_runner.tps() as f32));

        self.send_sim_query(SimQuery::Distances);
    }

    pub fn begin_run(&mut self) {
        if self.phase != DayPhase::Preparation {
            return;
        }
        self.phase = DayPhase::Running;

        self.send_sim_command(SimCommand::StartRun);
    }

    pub fn force_finish_day(&mut self) {
        if self.phase == DayPhase::Running {
            self.send_sim_command(SimCommand::EndRun);
        }
    }

    fn update_hud(&mut self) {
        if self.phase == DayPhase::Running {
            self.ui_commands.push(UiCommand::UpdateDayHud(
                DayHudState {
                    day_label: tr!("day-display.label", "day" = self.telemetry.day),
                    phase_label: tr!("phase-running.label"),
                    details: "Service running.".to_string(),
                    show_start: false,
                    enable_start: false,
                    show_dev: true,
                    enable_dev: true,
                }
                .into(),
            ));
        }

        self.ui_commands.push(UiCommand::UpdateStats(StatsView {
            sim_tick: self.telemetry.tick,
            sim_time: self.telemetry.seconds,
            fps: self.perf_tracker.last_fps,
            ups: self.perf_tracker.last_ups,
            current_diners: self.telemetry.live_diners,
            total_visits: self.telemetry.total_visits,
        }));
    }
}
