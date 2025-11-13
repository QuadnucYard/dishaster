use std::any::Any;

use dishaster_core::{interface::SimCommand, models::LevelSetupState, sim::Simulation};
use dishaster_godot_game::Game;
use dishaster_godot_ui::*;
use dishaster_ui_protocol::{AppRequest, GameRequest, PhaseMusic, UiCommand};
use dishrupt_core::asset::AudioRef;
use dishrupt_godot_input::event::GodotInputEvent;
use dishrupt_godot_scene::*;
use dishrupt_godot_ui::UITree;
use dishrupt_godot_utils::BindGodot;
use godot::{classes::Node, prelude::*};

use crate::{
    game_main::{ASSET_CATALOG, game_data},
    scenes::proc::*,
};

/// The in-game scene. Recreate the inner `Game` instance on each run.
pub struct GameScene {
    game: Option<Game>,

    gd: Gd<Node>,
}

impl BindGodot<Node> for GameScene {
    fn new(gd: Gd<Node>) -> Self {
        Self { game: None, gd }
    }
}

impl GameScene {
    pub const ID: SceneId = "game";
}

impl Scene for GameScene {
    fn id(&self) -> SceneId {
        Self::ID
    }

    fn gd(&self) -> Gd<Node> {
        self.gd.clone()
    }

    fn can_cache(&self) -> bool {
        false
    }

    fn enter(&mut self, ctx: &mut SceneContext) {
        let gui = &mut ctx.gui;

        gui.show::<GamingLayout>();
        gui.show::<TimeStatsGui>();
    }

    fn process(&mut self, ctx: &mut SceneContext, delta: f64) {
        if let Some(game) = self.game.as_mut() {
            game.process(delta);
        };

        ctx.gui_cmds.run_cmds(ctx.gui);

        for req in ctx.gui_cmds.take_reqs() {
            let req: Box<dyn Any> = req;
            match req.downcast::<GameRequest>() {
                Ok(req) => {
                    if let Some(game) = self.game.as_mut() {
                        Self::handle_game_request(ctx, *req, game);
                    }
                }
                Err(req) => {
                    let req = req.downcast::<AppRequest>().expect("app request");
                    Self::handle_app_request(ctx, *req);
                }
            }
        }

        // Process UI commands emitted by game logic
        if let Some(game) = self.game.as_mut() {
            for cmd in game.poll_ui_commands() {
                Self::handle_ui_command(ctx, cmd, game);
            }
        }
    }

    fn input(&mut self, _ctx: &mut SceneContext, event: GodotInputEvent) {
        let Some(game) = self.game.as_mut() else {
            return;
        };

        game.process_input(event);
    }
}

impl GameScene {
    pub fn start_game(&mut self, ctx: &mut SceneContext, level: LevelSetupState) {
        let catalog = ASSET_CATALOG.get().unwrap().clone();
        let db = &game_data().models;
        let mut game = Game::new(self.gd(), db.clone(), catalog, level, |level| {
            let mut sim = Box::new(Simulation::new(db.clone()));
            sim.start(level);
            sim
        });
        game.start_day();
        self.game = Some(game);

        ctx.gui.get_mut::<DishPricePopup>().enabled = true;
    }

    fn handle_app_request(ctx: &mut SceneContext, req: AppRequest) {
        match req {
            AppRequest::Quit
            | AppRequest::EnterLevel
            | AppRequest::ShowCredits
            | AppRequest::BackToMenu => {
                panic!("should be handled in main menu")
            }
            AppRequest::ExitLevel => {
                ctx.schedule(ExitLevelProcedure);
            }
        }
    }

    /// Handle a in-game ui request
    fn handle_game_request(ctx: &mut SceneContext, req: GameRequest, game: &mut Game) {
        match req {
            GameRequest::StartRun => {
                game.begin_run();
                ctx.gui.get_mut::<DishPricePopup>().enabled = false;
            }
            GameRequest::EndRun => {
                game.force_finish_day();
            }
            GameRequest::NextDay => {
                ctx.schedule(AdvanceLevelProcedure);
            }
            GameRequest::SetTps(tps) => {
                game.set_tps(tps);
                ctx.gui.get_mut::<TimeStatsGui>().set_tps_display(tps);
            }
            GameRequest::SetDebugMode(mode) => {
                game.set_debug_mode(mode);
            }

            GameRequest::ApplyDishPrice { dish, method } => {
                game.set_dish_price(dish, method);
            }

            GameRequest::TrialIntroDone => {
                godot_print!("Trial intro done");
                game.send_sim_command(SimCommand::TrialLaunch);
            }
            GameRequest::TrialCheckKeyword(keyword_index) => {
                godot_print!("Trial check keyword: {:?}", keyword_index);
                let trial_gui = ctx.gui.get_mut::<TrialGui>();
                trial_gui.check_keyword(keyword_index);
            }
            GameRequest::TrialBackFromThought => {
                let trial_gui = ctx.gui.get_mut::<TrialGui>();
                trial_gui.back_from_thought();
            }
            GameRequest::TrialRespond(corpus_index) => {
                godot_print!("Trial respond: {:?}", corpus_index);
                game.send_sim_command(SimCommand::TrialRespond(corpus_index));

                let trial_gui = ctx.gui.get_mut::<TrialGui>();
                trial_gui.finish_thought();
            }
            GameRequest::TrialResponseDone => {
                godot_print!("Trial response done");
                game.send_sim_command(SimCommand::TrialProceed);
            }
            GameRequest::TrialTimeout => {
                godot_print!("Trial timeout");
                game.send_sim_command(SimCommand::TrialTimeout);
            }

            GameRequest::ConfirmSettlement => {
                ctx.gui.hide::<SettlementGui>();
                ctx.gui.show::<ManageDecisionGui>();
            }
            GameRequest::SelectDecision(index) => {
                godot_print!("Decision selected: {}", index);
                game.send_sim_command(SimCommand::ApplyManagementDecision(index));

                ctx.gui.hide::<ManageDecisionGui>();
            }
            GameRequest::ConfirmIncident => {
                ctx.gui.hide::<ManageIncidentGui>();
            }
        }
    }

    /// Handle UI commands emitted by game logic.
    fn handle_ui_command(ctx: &mut SceneContext, cmd: UiCommand, game: &mut Game) {
        match cmd {
            UiCommand::FinishRun => {
                ctx.gui.hide::<GamingLayout>();
                ctx.gui.show::<SettlementGui>();
            }
            UiCommand::FinishDay => {
                ctx.schedule(AdvanceLevelProcedure);
            }

            UiCommand::UpdateTpsDisplay(tps) => {
                ctx.gui.get_mut::<TimeStatsGui>().set_tps_display(tps);
            }
            UiCommand::UpdateDayHud(state) => {
                ctx.gui.get_mut::<GamingLayout>().apply_state(&state);
            }
            UiCommand::UpdateStats(view) => {
                let stats_gui = ctx.gui.get_mut::<TimeStatsGui>();
                stats_gui.update_time(view.sim_tick, view.sim_time);
                stats_gui.update_perf(view.fps, view.ups);
                stats_gui.update_diner_stats(
                    view.current_diners,
                    view.total_visits,
                    view.completed_diners,
                    view.revenue,
                    view.consumption_kg,
                );
            }

            UiCommand::OpenDishPriceEditor(ref view) => {
                let popup = ctx.gui.get_mut::<DishPricePopup>();
                if popup.enabled {
                    popup.set_view(view);
                    popup.show();
                }
            }

            UiCommand::RefillDispenser(entity) => {
                game.send_sim_command(SimCommand::RefillDispenser(entity));
            }

            UiCommand::TrialStart(entity) => {
                godot_print!("Starting trial for entity {:?}", entity);
                game.send_sim_command(SimCommand::TrialStart(entity));
            }
            UiCommand::TrialIntro(intro) => {
                // Show trial GUI
                let trial_gui = ctx.gui.get_mut::<TrialGui>();
                trial_gui.intro(*intro);
                trial_gui.show();
            }
            UiCommand::TrialLeftSpeak(statement) => {
                let trial_gui = ctx.gui.get_mut::<TrialGui>();
                trial_gui.left_speak(*statement);
            }
            UiCommand::TrialRightSpeak(speech) => {
                let trial_gui = ctx.gui.get_mut::<TrialGui>();
                trial_gui.right_speak(*speech);
            }
            UiCommand::TrialEnd => {
                let trial_gui = ctx.gui.get_mut::<TrialGui>();
                trial_gui.hide();
            }

            UiCommand::ShowDecisionSelection(view) => {
                let catalog = ASSET_CATALOG.get().unwrap();
                ctx.gui
                    .get_mut::<ManageDecisionGui>()
                    .set_view(&view, catalog);
                // this command is emitted just after day ends
                // we defer showing the decision GUI until settlement is confirmed
            }
            UiCommand::ShowIncidentNotification(view) => {
                let catalog = ASSET_CATALOG.get().unwrap();
                ctx.gui
                    .get_mut::<ManageIncidentGui>()
                    .set_view(&view, catalog);
                ctx.gui.get_mut::<ManageIncidentGui>().show();
            }

            UiCommand::ShowHint { message } => {
                ctx.gui.get_mut::<HintNotification>().show_hint(&message);
            }

            // Audio commands - execute music playback via the scene's audio manager
            UiCommand::PlayPhaseMusic(phase) => {
                /// Cross-fade duration in seconds
                const FADE_DURATION: f32 = 2.0;

                let track = match phase {
                    PhaseMusic::Preparation => "canteen_preparation_theme",
                    PhaseMusic::Running => "canteen_running_theme",
                    PhaseMusic::Settlement => "canteen_settlement_theme",
                };

                ctx.audio
                    .play_music_crossfade(&AudioRef::new(track), FADE_DURATION);
            }
            UiCommand::EnterTrialMusic => {
                ctx.audio.pause_music();
                ctx.audio.play_music_loop(&AudioRef::new("trial_theme"));
            }
            UiCommand::ExitTrialMusic => {
                ctx.audio.stop_music();
                ctx.audio.resume_music();
            }
        }
    }
}
