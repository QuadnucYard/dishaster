use std::{any::Any, sync::LazyLock};

use dishaster_core::{
    models::Seed,
    views::{CreditSectionView, CreditsView},
};
use dishaster_godot_opening::Opening;
use dishaster_godot_ui::{CreditsGui, EndingGalleryGui, EndingGalleryView, StartMenuGui};
use dishaster_ui_protocol::AppRequest;
use dishrupt_core::asset::{AudioRef, PrefabRef};
use dishrupt_godot_audio::AudioManager;
use dishrupt_godot_input::event::GodotInputEvent;
use dishrupt_godot_scene::{Scene, SceneContext, SceneId};
use dishrupt_godot_ui::{GuiCommands, GuiRegistry};
use dishrupt_godot_utils::BindGodot;
use godot::{
    classes::Node,
    global::{godot_error, godot_print},
    obj::Gd,
};

use crate::{effect::pend_effect, game_main::game_services, scenes::proc::EnterLevelProcedure};

static MAIN_THEME_MUSIC: LazyLock<AudioRef> = LazyLock::new(|| AudioRef::new("main_theme.ogg"));

/// The root scene. Handles interaction outside levels.
pub struct StartScene {
    opening: Option<Opening>,

    gd: Gd<Node>,
}

impl BindGodot<Node> for StartScene {
    fn new(gd: Gd<Node>) -> Self {
        Self { opening: None, gd }
    }
}

impl StartScene {
    pub const ID: SceneId = "start";
}

impl Scene for StartScene {
    fn id(&self) -> SceneId {
        Self::ID
    }

    fn gd(&self) -> Gd<Node> {
        self.gd.clone()
    }

    fn enter(&mut self, ctx: &mut SceneContext) {
        let gui = ctx.res.get_mut::<GuiRegistry>();

        let services = game_services();

        gui.show::<StartMenuGui>();

        // Update toggle buttons from saved settings
        {
            let svc = &services.user_service.prefs;
            let audio_prefs = &svc.load().expect("failed to load prefs").audio;
            gui.get_mut::<StartMenuGui>()
                .update_from_preferences(audio_prefs.music_mute, audio_prefs.sound_mute);
        }

        // Update ending gallery buttons based on unlocked endings
        {
            let profile = services
                .user_service
                .profiles
                .load()
                .expect("failed to load profile");
            gui.get_mut::<StartMenuGui>()
                .update_endings_unlocked(&profile.achieved_endings.iter().cloned().collect());
        }

        if self.opening.is_none() {
            let catalog = services.catalog.clone();
            let config = services.data.opening_config.clone();
            let opening = Opening::new(self.gd.clone(), config, catalog);
            self.opening = Some(opening);
        }

        let audio = ctx.res.get_mut::<AudioManager>();
        audio.play_music(&MAIN_THEME_MUSIC);
    }

    fn leave(&mut self, ctx: &mut SceneContext) {
        let gui = ctx.res.get_mut::<GuiRegistry>();
        gui.hide_all();
    }

    fn process(&mut self, ctx: &mut SceneContext, delta: f64) {
        if let Some(opening) = self.opening.as_mut() {
            opening.process(delta);
        };

        let (gui, gui_cmds) = ctx.res.get_many_mut::<(GuiRegistry, GuiCommands)>();

        gui_cmds.run_cmds(gui);

        for req in gui_cmds.take_reqs() {
            let req: Box<dyn Any> = req;

            // Do not panic if request is not an `AppRequest` — log and ignore unknown types.
            match req.downcast::<AppRequest>() {
                Ok(req) => self.handle_app_request(ctx, *req),
                Err(_) => {
                    godot_error!("StartScene received unknown GUI request — expected AppRequest");
                }
            }
        }
    }

    fn input(&mut self, _ctx: &mut SceneContext, event: GodotInputEvent) {
        if let GodotInputEvent::Button(e) = event
            && e.pressed
        {
            pend_effect(PrefabRef::new("heart_break"), None);
        }
    }
}

impl StartScene {
    fn handle_app_request(&mut self, ctx: &mut SceneContext, req: AppRequest) {
        let (gui, audio) = ctx.res.get_many_mut::<(GuiRegistry, AudioManager)>();

        match req {
            AppRequest::Quit => {
                godot_print!("Quit requested");
                self.gd.get_tree().expect("failed to get scene tree").quit();
            }
            AppRequest::EnterLevel => {
                ctx.schedule(EnterLevelProcedure);
            }
            AppRequest::ShowCredits => {
                let credits_data = &game_services().data.credits;

                let credits_view = CreditsView {
                    sections: credits_data
                        .sections
                        .iter()
                        .map(|s| CreditSectionView {
                            title: s.title.clone(),
                            entries: s.entries.clone(),
                        })
                        .collect(),
                };

                gui.hide::<StartMenuGui>();
                gui.get_mut::<CreditsGui>().set_view(credits_view);
                gui.show::<CreditsGui>();
            }
            AppRequest::ViewEnding(ending_id) => {
                godot_print!("Viewing ending: {:?}", ending_id);

                if let Some(ending_model) = game_services().data.endings.get(&ending_id) {
                    let ending_view = EndingGalleryView {
                        id: ending_id.clone(),
                        illustration: ending_model.illustration.clone(),
                    };

                    gui.get_mut::<EndingGalleryGui>()
                        .show_ending(ending_view, &game_services().catalog);
                } else {
                    godot_error!("Requested unknown ending ID: {}", ending_id);
                }
            }
            AppRequest::BackToMenu => {
                gui.hide::<CreditsGui>();
                gui.hide::<EndingGalleryGui>();
                gui.show::<StartMenuGui>();
            }

            AppRequest::ToggleMusic(mute) => {
                godot_print!("Toggling music: {}", mute);
                // Apply immediately
                audio.set_music_mute(mute);

                let svc = &game_services().user_service.prefs;
                let res = svc.update(|prefs| {
                    prefs.audio.music_mute = mute;
                    Ok(())
                });
                if let Err(e) = res {
                    godot_error!("Failed to save preferences: {}", e);
                }
            }
            AppRequest::ToggleSound(mute) => {
                // Apply immediately
                audio.set_sound_mute(mute);

                let svc = &game_services().user_service.prefs;
                let res = svc.update(|prefs| {
                    prefs.audio.sound_mute = mute;
                    Ok(())
                });
                if let Err(e) = res {
                    godot_error!("Failed to save preferences: {}", e);
                }
            }
            AppRequest::RollSeed => {
                godot_print!("Rolling new seed for player profile");
                let new_seed = godot::global::randi() as u64;
                let svc = &game_services().user_service.profiles;
                if let Err(e) = svc.update(|profile| {
                    if let Some(level_progress) = &mut profile.level_progress {
                        level_progress.rng_seed = Seed::new(new_seed);
                    }
                    Ok(())
                }) {
                    godot_error!("Failed to roll new seed: {e}");
                } else {
                    godot_print!("New seed rolled successfully: {new_seed}");
                }
            }
            AppRequest::ClearLevel => {
                godot_print!("Deleting player profile");
                let svc = &game_services().user_service.profiles;
                if let Err(e) = svc.clear_level_progress() {
                    godot_error!("Failed to delete profile: {}", e);
                } else {
                    godot_print!("Profile deleted successfully");
                }
            }
            AppRequest::DeleteProfile => {
                godot_print!("Deleting player profile");
                let svc = &game_services().user_service.profiles;
                if let Err(e) = svc.delete() {
                    godot_error!("Failed to delete profile: {}", e);
                } else {
                    godot_print!("Profile deleted successfully");
                    // Refresh the UI to show default state
                    gui.get_mut::<StartMenuGui>()
                        .update_endings_unlocked(&Default::default());
                }
            }
            AppRequest::ExitLevel => panic!("should not happen in start menu"),

            AppRequest::SpawnEffectAtMouse(prefab) => {
                pend_effect(prefab, None);
            }
        }
    }
}
