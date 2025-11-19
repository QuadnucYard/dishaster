//! Dishaster Godot Game Module

mod dbgviz;
mod handle_request;
mod hint;
mod input;
pub mod perf;
pub mod persist;
mod present;

use std::sync::Arc;

use dishaster_interface::*;
use dishaster_models::{GameModelRegistry, LevelSetupState};
use dishaster_persistence::ProfileService;
use dishaster_ui_protocol::{PhaseMusic, StatsView, UiCommand};
use dishaster_views::DayHudState;
use dishrupt_asset::AssetCatalog;
use dishrupt_core::prelude::*;
use dishrupt_godot_display::{DisplayContext2D, GdNode2D, Stage, load_prefab_sync};
use dishrupt_godot_utils::NodeExt;
use dishrupt_l10n::tr;
use dishrupt_runner::{ISimulation, SimulationRunner, SnapshotFrame, SyncSimulationRunner};
use godot::{
    classes::{Node, Node2D},
    prelude::*,
};
use rustc_hash::FxHashMap;

use self::{perf::PerfTracker, present::*};
use crate::{dbgviz::*, hint::HintTracker};

const DEFAULT_TPS: f64 = 120.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DayPhase {
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
    world_time: f64,
    /// Current number of live diners in the canteen
    live_diners: u32,
    /// Total number of diners that have visited this day
    total_visits: u32,
    /// Number of diners who completed their meal today
    completed_diners: u32,
    /// Total revenue collected today
    revenue: f32,
    /// Total food consumed today in kilograms
    consumption_kg: f32,
}

pub struct Game {
    root: Gd<Node>,
    map_root: Gd<Node>,

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
    dev_features_enabled: bool,
    suspended_sim_speed: Option<f64>,

    profile_svc: Arc<ProfileService>,
    asset_catalog: Arc<AssetCatalog>,

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
        db: Arc<GameModelRegistry>,
        asset_catalog: Arc<AssetCatalog>,
        profile_svc: Arc<ProfileService>,
        level: LevelSetupState,
        sim_creator: impl FnOnce(LevelSetupState) -> Box<dyn ISimulation<CoreSimulationFeat>>,
    ) -> Self {
        let level_config = db
            .levels
            .get_by_id(&level.level_id)
            .expect("failed to get level");

        let telemetry = DayTelemetry {
            seed: level.seed.get(),
            day: level.day.0,
            world_time: level_config.entry_time as f64,
            ..Default::default()
        };

        let map_prefab = &db
            .canteens
            .get_by_id(&level_config.canteen)
            .expect("failed to get canteen")
            .display
            .res;

        // Initialize simulation
        let sim = sim_creator(level);
        let root_entity = sim.root_entity();
        let sim_runner = SyncSimulationRunner::new(sim, DEFAULT_TPS);

        // Set up the map scene
        let mut stage_root = gd.get_node_as::<Node2D>("%Stage");
        let map_scene = load_prefab_sync(map_prefab, &asset_catalog)
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
            view_scale: Vec3::new(50.0, 40.0, 40.0),
        };
        let mut stage = Stage::new(display_ctx, asset_catalog.clone());
        stage.set_root(root_entity, GdNode2D::new(display_root_node));

        // Set up debug visualization
        let mut dbgviz = {
            let mut debug_root = stage_root.get_or_add_node_as::<Node2D>("Debug");
            debug_root.set_position(origin);
            DbgViz::new(debug_root)
        };
        dbgviz.set_visible(false);

        Self {
            root: gd,
            map_root: map_scene,
            sim_runner: Box::new(sim_runner),
            stage,
            stage_origin: origin,
            dbgviz,

            perf_tracker: Default::default(),
            pres: Default::default(),
            hint_tracker: HintTracker::new(profile_svc.load().unwrap().seen_hints.clone()),

            phase: DayPhase::Preparation,
            telemetry,
            debug_enabled: false,
            dev_features_enabled: false,
            suspended_sim_speed: None,

            profile_svc,
            asset_catalog,
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
            self.telemetry.world_time = snapshot.stats.world_time;
            self.telemetry.live_diners = snapshot.stats.live_diners;
            self.telemetry.total_visits = snapshot.stats.total_visits;
            self.telemetry.consumption_kg = snapshot.stats.consumption_kg;
            self.telemetry.revenue = snapshot.stats.revenue;
            self.telemetry.completed_diners = snapshot.stats.completed_diners;
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
        // Emit command to start preparation music
        self.ui_commands
            .push(UiCommand::PlayPhaseMusic(PhaseMusic::Preparation));

        self.ui_commands.push(UiCommand::UpdateDayHud(
            DayHudState {
                day_label: tr!("day-display.label", "day" = self.telemetry.day),
                phase_label: tr!("phase-preparation.label"),
                details: tr!("phase-preparation.desc"),
                show_start: true,
                enable_start: true,
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

        // Emit command to transition to running phase music
        self.ui_commands
            .push(UiCommand::PlayPhaseMusic(PhaseMusic::Running));

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
                    details: tr!("phase-running.desc"),
                    show_start: false,
                    enable_start: false,
                }
                .into(),
            ));
        }

        let view = Box::new(StatsView {
            sim_tick: self.telemetry.tick,
            sim_time: self.telemetry.seconds,
            world_time: self.telemetry.world_time,
            fps: self.perf_tracker.last_fps,
            ups: self.perf_tracker.last_ups,
            current_diners: self.telemetry.live_diners,
            total_visits: self.telemetry.total_visits,
            consumption_kg: self.telemetry.consumption_kg,
            revenue: self.telemetry.revenue,
            completed_diners: self.telemetry.completed_diners,
        });
        self.ui_commands.push(UiCommand::UpdateStats(view));
    }

    fn set_dev_enabled(&mut self, enabled: bool) {
        self.dev_features_enabled = enabled;
        self.ui_commands.push(UiCommand::ToggleDev(enabled));

        godot_print!(
            "DEV: Dev features {}",
            if self.dev_features_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}
