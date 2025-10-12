mod agent;
mod dish;
pub mod perf;
mod present;

mod controllers {
    pub use super::{agent::AgentController, dish::DishController};
}

use dishaster_core::{commands::SimCommand, models::LevelConfig, sim::Simulation};
use dishaster_godot_ui::*;
use dishrupt_core::{EntityId, prelude::*};
use dishrupt_godot::{display::*, input::listener::GodotInputEvent};
use dishrupt_godot_scene::SceneContext;
use dishrupt_l10n_godot::tr;
use godot::{
    classes::{Node, Node2D},
    global::MouseButton,
    prelude::*,
};
use rustc_hash::FxHashMap;

use self::{controllers::*, perf::PerfTracker};
use crate::{
    dbgviz::*,
    game_main::{GAME_DATA, progress_service},
    runner::{SnapshotFrame, SyncSimulationRunner},
};

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

    sim_runner: SyncSimulationRunner,
    stage: Stage,
    stage_origin: Vector2,
    display_ctx: DisplayContext2D,
    dbgviz: DbgViz,

    perf_tracker: PerfTracker,
    dc: DisplayControllers,

    phase: DayPhase,
    telemetry: DayTelemetry,
}

#[derive(Default)]
struct DisplayControllers {
    agents: FxHashMap<EntityId, AgentController>,
    dishes: FxHashMap<EntityId, DishController>,
}

impl Game {
    pub fn new(gd: Gd<Node>, level: LevelConfig) -> Self {
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
        let mut sim = Simulation::new(db.clone());
        sim.start(level);
        let root_entity = sim.root_entity();
        let default_tps = 60.0;
        let sim_runner = SyncSimulationRunner::new(sim, default_tps);

        // Set up stage
        let mut stage = Stage::new();
        stage.set_display_context(display_ctx.clone());
        stage.set_root(root_entity, display_root.clone());

        // Set up debug visualization
        let dbgviz = DbgViz::new(&stage_root, origin);

        Self {
            root: gd,
            sim_runner,
            stage,
            stage_origin: origin,
            display_ctx,
            dbgviz,
            perf_tracker: Default::default(),
            dc: Default::default(),
            phase: DayPhase::Preparation,
            telemetry,
        }
    }

    pub fn process(&mut self, delta: f64, ctx: &mut SceneContext) {
        self.perf_tracker.tick_frame();

        if let Some(SnapshotFrame {
            ticks,
            snapshot,
            events,
        }) = self.sim_runner.tick(delta)
        {
            self.perf_tracker.tick_updates(ticks);

            self.stage.present(snapshot.display.iter());
            self.dbgviz.update(&snapshot, &self.display_ctx);

            self.process_events(ctx, events);
            self.telemetry.tick = snapshot.sim_tick;
            self.telemetry.seconds = snapshot.sim_time_seconds;
        }

        self.process_display(delta);
        self.perf_tracker.sample(delta);
        self.update_hud(ctx);
    }

    pub fn process_input(&mut self, event: GodotInputEvent) {
        #[allow(clippy::single_match)]
        match event {
            GodotInputEvent::Button(e) => {
                if e.button == MouseButton::LEFT && !e.pressed {
                    let canvas_pos = screen_to_canvas(&self.root, e.position);
                    let sim_pos = self.to_map_pos(canvas_pos);
                    godot_print!("click map： {canvas_pos} {sim_pos}");
                    self.sim_runner
                        .send_command(SimCommand::QueryDistance(sim_pos));
                }
            }
            _ => {}
        }
    }

    fn to_map_pos(&self, pos: Vector2) -> Vec2 {
        self.display_ctx
            .to_simulation_space(pos - self.stage_origin)
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

        self.sim_runner.send_command(SimCommand::QueryDistances);
    }

    pub fn begin_run(&mut self, _ctx: &mut SceneContext) {
        if self.phase != DayPhase::Preparation {
            return;
        }
        self.phase = DayPhase::Running;

        self.sim_runner.send_command(SimCommand::StartRun);
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
            self.sim_runner.send_command(SimCommand::EndRun);
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

    /// Update the simulation tick rate and refresh related UI state.
    pub fn set_tps(&mut self, ctx: &mut SceneContext, requested_tps: f32) {
        if (self.sim_runner.tps() - requested_tps as f64).abs() <= f64::EPSILON {
            return;
        }

        self.sim_runner.set_tps(requested_tps as f64);

        ctx.gui
            .get_mut::<TimeStatsGui>()
            .set_tps_display(requested_tps);
    }
}

fn screen_to_canvas(root: &Gd<Node>, screen_pos: Vector2) -> Vector2 {
    root.get_viewport()
        .unwrap()
        .get_canvas_transform()
        .affine_inverse()
        * screen_pos
}
