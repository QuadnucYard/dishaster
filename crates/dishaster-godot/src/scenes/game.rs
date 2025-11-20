use std::{any::Any, sync::Arc};

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
    effect::pend_effect,
    game_main::{GameServices, game_services},
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
        let SceneContext { gui, .. } = ctx;

        gui.get_mut::<GamingLayout>().set_dev_enabled(false);
        gui.get_mut::<TimeStatsGui>().set_dev_enabled(false);

        gui.show::<GamingLayout>();
        gui.show::<TimeStatsGui>();
        gui.show::<TrialImpactGui>();
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
    pub fn start_game(
        &mut self,
        ctx: &mut SceneContext,
        level: LevelSetupState,
        services: Arc<GameServices>,
    ) {
        let db = services.data.models.clone();
        let catalog = services.catalog.clone();
        let profile_svc = services.user_service.profiles.clone();
        let mut game = Game::new(
            self.gd(),
            db.clone(),
            catalog,
            profile_svc,
            level,
            |level| {
                let mut sim = Box::new(Simulation::new(db));
                sim.start(level);
                sim
            },
        );
        game.start_day();
        self.game = Some(game);

        ctx.gui.get_mut::<DishPricePopup>().enabled = true;
    }

    fn handle_app_request(ctx: &mut SceneContext, req: AppRequest) {
        match req {
            AppRequest::Quit
            | AppRequest::EnterLevel
            | AppRequest::ShowCredits
            | AppRequest::BackToMenu
            | AppRequest::ViewEnding(_)
            | AppRequest::RollSeed
            | AppRequest::ClearLevel
            | AppRequest::DeleteProfile => {
                godot_error!("AppRequest should be handled in main menu");
            }
            AppRequest::ExitLevel => {
                ctx.schedule(ExitLevelProcedure);
            }
            AppRequest::ToggleMusic(_mute) => {}
            AppRequest::ToggleSound(_mute) => {}
            AppRequest::SpawnEffectAtMouse(prefab) => {
                pend_effect(prefab, None);
            }
        }
    }

    /// Handle a in-game ui request
    fn handle_game_request(ctx: &mut SceneContext, req: GameRequest, game: &mut Game) {
        let SceneContext { gui, .. } = ctx;

        match req {
            GameRequest::StartRun => {
                game.begin_run();
                gui.get_mut::<DishPricePopup>().enabled = false;
                gui.show::<ReputationGui>();
            }
            GameRequest::EndRun => {
                game.force_finish_day();
            }
            GameRequest::NextDay => {
                ctx.schedule(AdvanceLevelProcedure);
            }
            GameRequest::SetTps(tps) => {
                game.set_tps(tps);
                gui.get_mut::<TimeStatsGui>().set_tps_display(tps);
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
            GameRequest::TrialCheckKeyword {
                speech_id,
                keyword_index,
            } => {
                godot_print!(
                    "Trial check keyword: speech={}, keyword={}",
                    speech_id,
                    keyword_index
                );
                // Send command to simulation to generate candidates
                game.send_sim_command(SimCommand::TrialRequestCandidates {
                    speech_id,
                    keyword_index,
                });
            }
            GameRequest::TrialBackFromThought => {
                let trial_gui = gui.get_mut::<TrialGui>();
                trial_gui.back_from_thought();
            }
            GameRequest::TrialRespond(corpus_index) => {
                godot_print!("Trial respond: {:?}", corpus_index);
                game.send_sim_command(SimCommand::TrialRespond(corpus_index));

                let trial_gui = gui.get_mut::<TrialGui>();
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
                // Send command to simulation to check endings and roll decisions
                game.send_sim_command(SimCommand::ConfirmSettlement);
                gui.hide::<SettlementGui>();
            }
            GameRequest::ContinueFromEnding => {
                // Hide ending screen and show decision selection
                gui.hide::<EndingGui>();

                // Send command to simulation to roll management decisions
                game.send_sim_command(SimCommand::ContinueFromEnding);
            }
            GameRequest::SelectDecision(index) => {
                godot_print!("Decision selected: {}", index);
                game.send_sim_command(SimCommand::ApplyManagementDecision(index));

                gui.hide::<ManageDecisionGui>();
            }

            GameRequest::ClearLevel => {
                // Clear level progress only when exiting after an ending
                godot_print!("Clearing current level progress");
                let svc = &game_services().user_service.profiles;
                if let Err(e) = svc.clear_level_progress() {
                    godot_error!("Failed to clear level progress: {e}");
                } else {
                    godot_print!("Level progress cleared successfully");
                }
            }
        }
    }

    /// Handle UI commands emitted by game logic.
    fn handle_ui_command(ctx: &mut SceneContext, cmd: UiCommand, game: &mut Game) {
        let SceneContext { gui, audio, .. } = ctx;

        match cmd {
            UiCommand::ToggleDev(enabled) => {
                gui.get_mut::<GamingLayout>().set_dev_enabled(enabled);
                gui.get_mut::<TimeStatsGui>().set_dev_enabled(enabled);
            }

            UiCommand::FinishRun => {
                gui.hide::<GamingLayout>();
                gui.hide::<ReputationGui>();
            }
            UiCommand::ShowSettlement(view) => {
                gui.get_mut::<SettlementGui>().set_view(&view);
                gui.show::<SettlementGui>();
            }
            UiCommand::FinishDay => {
                ctx.schedule(AdvanceLevelProcedure);
            }

            UiCommand::UpdateTpsDisplay(tps) => {
                gui.get_mut::<TimeStatsGui>().set_tps_display(tps);
            }
            UiCommand::UpdateDayHud(state) => {
                gui.get_mut::<GamingLayout>().apply_state(&state);
            }
            UiCommand::UpdateStats(view) => {
                let stats_gui = gui.get_mut::<TimeStatsGui>();
                stats_gui.update_time(view.sim_tick, view.sim_time, view.world_time);
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
                let popup = gui.get_mut::<DishPricePopup>();
                if popup.enabled {
                    popup.set_view(view);
                    popup.show();
                }
            }

            UiCommand::RefillDispenser(entity) => {
                game.send_sim_command(SimCommand::RefillDispenser(entity));
            }

            UiCommand::TrialStart { diner, topic } => {
                godot_print!(
                    "Starting trial for entity {:?} with topic {:?}",
                    diner,
                    topic
                );
                game.send_sim_command(SimCommand::TrialStart { diner, topic });
            }
            UiCommand::TrialIntro(intro) => {
                // Show trial GUI
                let trial_gui = gui.get_mut::<TrialGui>();
                trial_gui.intro(*intro);
                trial_gui.show();
            }
            UiCommand::TrialLeftSpeak(statement) => {
                let trial_gui = gui.get_mut::<TrialGui>();
                trial_gui.left_speak(*statement);
            }
            UiCommand::TrialRightSpeak(statement) => {
                let trial_gui = gui.get_mut::<TrialGui>();
                trial_gui.right_speak(*statement);
            }
            UiCommand::TrialResponseCandidates(options) => {
                let trial_gui = gui.get_mut::<TrialGui>();
                trial_gui.show_response_candidates(options);
            }
            UiCommand::TrialImpact(impact) => {
                let trial_impact_gui = gui.get_mut::<TrialImpactGui>();
                trial_impact_gui.show_impact(*impact);
            }
            UiCommand::TrialEnd { timeout: _timeout } => {
                let trial_gui = gui.get_mut::<TrialGui>();
                trial_gui.hide();
            }

            UiCommand::ShowDecisionSelection(view) => {
                // Show decision GUI after settlement confirmation (or after good ending)
                let catalog = &game_services().catalog;
                gui.get_mut::<ManageDecisionGui>().set_view(&view, catalog);
                gui.show::<ManageDecisionGui>();
            }
            UiCommand::ShowIncidentNotification(view) => {
                let catalog = &game_services().catalog;
                gui.get_mut::<ManageIncidentGui>().set_view(&view, catalog);
                gui.get_mut::<ManageIncidentGui>().show();
            }
            UiCommand::ShowInspectorResult(view) => {
                gui.get_mut::<InspectorResultGui>().set_view(&view);
                gui.get_mut::<InspectorResultGui>().show();
            }

            UiCommand::ShowTutorial => {
                gui.show::<TutorialGui>();
            }

            UiCommand::ShowEnding(ending) => {
                if let Some(ending_model) = game_services().data.endings.get(&ending.id) {
                    gui.get_mut::<EndingGui>()
                        .set_ending_picture(&ending_model.illustration, &game_services().catalog);
                } else {
                    godot_error!("Requested unknown ending ID: {}", ending.id);
                }
                gui.get_mut::<EndingGui>().show_ending(*ending);
            }

            UiCommand::ShowHint { message } => {
                gui.get_mut::<HintNotification>().show_hint(&message);
            }

            UiCommand::UpdateReputation(view) => {
                gui.get_mut::<ReputationGui>().update(&view);
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

                audio.play_music_crossfade(&AudioRef::new(track), FADE_DURATION);
            }
            UiCommand::EnterTrialMusic => {
                audio.pause_music();
                audio.play_music_loop(&AudioRef::new("trial_theme"));
            }
            UiCommand::ExitTrialMusic => {
                audio.stop_music();
                audio.resume_music();
            }
        }
    }
}
