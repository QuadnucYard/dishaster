use std::{
    cell::OnceCell,
    path::Path,
    sync::{Arc, OnceLock},
};

use anyhow::{Context, Result};
use dishaster_data::{DataLoader, GameDataAssets, load_toml};
use dishaster_godot_ui::register_guis;
use dishaster_persistence::UserDataService;
use dishrupt_asset::{AssetCatalog, AssetPathConfig, AssetResolver};
use dishrupt_godot_audio::AudioManager;
use dishrupt_godot_input::listener::InputListener;
use dishrupt_godot_scene::{SceneContext, SceneManager};
use dishrupt_godot_ui::GuiManager;
use dishrupt_godot_utils::NodeExt;
use dishrupt_l10n_godot::LocalizationManager;
use dishrupt_persistence::GodotUserStorage;
use godot::{classes::CanvasLayer, prelude::*};

use crate::{
    panic::{get_panic_message, has_panic_occurred, init_backtrace_handle},
    panic_overlay::PanicOverlay,
    scenes::{DefaultSceneLoader, proc::*},
};

/// The root scene. Handles interaction outside levels.
#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct GameMain {
    inner: OnceCell<Inner>,

    base: Base<Node>,
}

/// For godot nodes obtained after initialization.
struct Inner {
    scene_manager: SceneManager,
    gui: GuiManager,
    audio: AudioManager,
    l10n: LocalizationManager,
    input_listener: Gd<InputListener>,
    panic_overlay: PanicOverlay,

    services: Arc<GameServices>,

    late_initialized: bool,
    panic_displayed: bool,
}

#[godot_api]
impl INode for GameMain {
    /// The game entry.
    fn ready(&mut self) {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init();

        log::info!("Main loop initialize");

        match std::panic::catch_unwind(init_game) {
            Ok(Ok(services)) => {
                log::info!("Init game completed successfully");

                GAME_SERVICES
                    .set(services.clone())
                    .unwrap_or_else(|_| panic!("failed to set global game services"));

                let mut inner = Inner::new(self.base().clone().upcast(), services);
                inner.ready();
                let _ = self.inner.set(inner);
            }
            Ok(Err(e)) => {
                log::error!("Error during init_game: {:?}", e);
                godot_error!("Error during init_game: {:?}", e);
            }
            Err(e) => {
                log::error!("Panic during init_game: {:?}", e);
                godot_error!("Panic during init_game: {:?}", e);
            }
        }
    }

    fn process(&mut self, delta: f64) {
        if let Some(inner) = self.inner.get_mut() {
            inner.process(delta);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if let Some(inner) = self.inner.get_mut() {
            inner.physics_process(delta);
        }
    }
}

impl Inner {
    fn new(mut root: Gd<Node>, services: Arc<GameServices>) -> Inner {
        let scene_root = root.get_or_add_node_as::<Node2D>("SceneRoot");
        let gui_root = root.get_or_add_node_as::<CanvasLayer>("UIRoot");
        let audio_root = root.get_or_add_node_as("AudioRoot");

        let scene_manager = SceneManager::new(
            scene_root.upcast(),
            DefaultSceneLoader::new(services.catalog.clone()),
        );
        let gui = GuiManager::new(gui_root.upcast());
        let audio = AudioManager::new(audio_root, services.catalog.clone());

        let l10n = Default::default();
        let input_listener = root.get_or_add_node_of_type::<InputListener>();

        let panic_overlay = PanicOverlay::new(root.get_node_as("%PanicOverlay"));

        Self {
            scene_manager,
            gui,
            audio,
            l10n,
            input_listener,
            panic_overlay,

            services,

            late_initialized: false,
            panic_displayed: false,
        }
    }

    fn ready(&mut self) {
        register_guis(&mut self.gui.registry, &self.services.catalog);
        self.gui.ready();

        self.apply_preferences();

        let profile_svc = &self.services.user_service.profiles;
        if dishaster_validation::validate_player_profile(
            &profile_svc.load().expect("failed to load profile"),
            &self.services.data.models,
        )
        .is_err()
        {
            log::warn!("Player profile validation failed, resetting to default profile");
            profile_svc.create().expect("failed to recreate profiles");
        };

        self.scene_manager.schedule(StartProcedure);
    }

    fn ready_late(&mut self) {
        godot_print!("collect localized elements...");
        self.l10n.collect(self.gui.root.gd());
        self.l10n.update();
    }

    fn process(&mut self, delta: f64) {
        if has_panic_occurred() {
            if self.panic_displayed {
                return;
            }
            self.panic_displayed = true;
            if let Some(message) = get_panic_message() {
                self.panic_overlay.show_panic(&message);
            } else {
                self.panic_overlay
                    .show_panic("A panic occurred but no message was captured.");
            }
            return;
        }
        init_backtrace_handle();

        if !self.late_initialized {
            self.ready_late();
            self.late_initialized = true;
        }

        {
            let ctx = &mut SceneContext {
                gui: &mut self.gui.registry,
                gui_cmds: self.gui.cmds.clone(),
                audio: &mut self.audio,
                proc: None,
            };

            // process scene events
            self.scene_manager.process(ctx);
        }

        // process UI
        self.gui.process(delta);

        // process active scene
        {
            let ctx = &mut SceneContext {
                gui: &mut self.gui.registry,
                gui_cmds: self.gui.cmds.clone(),
                audio: &mut self.audio,
                proc: None,
            };
            self.scene_manager.inspect_active_scene_mut(|scene| {
                scene.process(ctx, delta);
            });
            if let Some(proc) = ctx.proc.take() {
                self.scene_manager.schedule1(proc);
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if has_panic_occurred() {
            return;
        }
        init_backtrace_handle();

        self.scene_manager.inspect_active_scene_mut(|scene| {
            let ctx = &mut SceneContext {
                gui: &mut self.gui.registry,
                gui_cmds: self.gui.cmds.clone(),
                audio: &mut self.audio,
                proc: None,
            };

            self.input_listener
                .bind_mut()
                .drain_events()
                .for_each(|e| scene.input(ctx, e));

            scene.physics_process(ctx, delta);
        });
    }
}

impl Inner {
    fn apply_preferences(&mut self) {
        let prefs = self
            .services
            .user_service
            .prefs
            .load()
            .expect("failed to load preferences");
        let audio_prefs = &prefs.audio;

        self.audio.set_music_mute(audio_prefs.music_mute);
        self.audio.set_sound_mute(audio_prefs.sound_mute);
        self.audio.set_music_volume(audio_prefs.music_volume);
        self.audio.set_sound_volume(audio_prefs.sound_volume);
    }
}

fn init_game() -> Result<Arc<GameServices>> {
    log::info!("Init game start");

    let assets_path_config =
        load_toml::<AssetPathConfig>(Path::new("assets.toml")).context("loading assets.toml")?;
    println!("Loaded assets config: {assets_path_config:#?}");
    let catalog = Arc::new(AssetCatalog::new(
        Arc::new(assets_path_config),
        AssetResolver,
    ));

    let data = Arc::new(
        DataLoader::new_with_fallback("data", "../assets/data")
            .context("failed to create data loader")?
            .load_all_data()
            .context("failed to load game data")?,
    );

    let user_service = Arc::new(UserDataService::new(Arc::new(GodotUserStorage)));

    dishrupt_l10n_godot::init();
    log::info!("Localization initialized");

    Ok(Arc::new(GameServices {
        catalog,
        data,
        user_service,
    }))
}

pub struct GameServices {
    pub catalog: Arc<AssetCatalog>,
    pub data: Arc<GameDataAssets>,
    pub user_service: Arc<UserDataService>,
}

static GAME_SERVICES: OnceLock<Arc<GameServices>> = OnceLock::new();

/// Before we can inject services into scenes, we need a way to access them globally.
pub(crate) fn game_services() -> &'static Arc<GameServices> {
    GAME_SERVICES.get().expect("game data not initialized")
}
