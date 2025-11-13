use std::{
    cell::OnceCell,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::{Context, Result, anyhow};
use dishaster_data::{DataLoader, GameDataAssets, load_toml};
use dishaster_godot_game::{PROGRESS_SERVICE, user_store::GodotUserStorage};
use dishaster_godot_ui::register_guis;
use dishaster_persistence::PlayerService;
use dishrupt_asset::{AssetCatalog, AssetPathConfig, AssetResolver};
use dishrupt_godot_audio::AudioManager;
use dishrupt_godot_input::listener::InputListener;
use dishrupt_godot_scene::{SceneContext, SceneManager};
use dishrupt_godot_ui::GuiManager;
use dishrupt_godot_utils::NodeExt;
use dishrupt_l10n_godot::LocalizationManager;
use godot::{classes::CanvasLayer, prelude::*};

use crate::{
    panic::{has_panic_occurred, init_backtrace_handle},
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

    late_initialized: bool,
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
            Ok(_) => {
                log::info!("Init game completed successfully");

                let mut inner = Inner::new(self.base().clone().upcast());
                inner.ready();
                let _ = self.inner.set(inner);
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
    fn new(mut root: Gd<Node>) -> Inner {
        let scene_root = root.get_or_add_node_as::<Node2D>("SceneRoot");
        let gui_root = root.get_or_add_node_as::<CanvasLayer>("UIRoot");
        let audio_root = root.get_or_add_node_as("AudioRoot");

        let input_listener = root.get_or_add_node_of_type::<InputListener>();

        let catalog = ASSET_CATALOG.get().unwrap().clone();

        Self {
            scene_manager: SceneManager::new(
                scene_root.upcast(),
                DefaultSceneLoader::new(catalog.clone()),
            ),
            gui: GuiManager::new(gui_root.upcast()),
            audio: AudioManager::new(audio_root, catalog),
            l10n: Default::default(),
            input_listener,

            late_initialized: false,
        }
    }

    fn ready(&mut self) {
        register_guis(
            &mut self.gui.registry,
            &ASSET_CATALOG.get().unwrap().clone(),
        );
        self.gui.ready();

        self.scene_manager.schedule(StartProcedure);
    }

    fn ready_late(&mut self) {
        godot_print!("collect localized elements...");
        self.l10n.collect(self.gui.root.gd());
        self.l10n.update();
    }

    fn process(&mut self, delta: f64) {
        if has_panic_occurred() {
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

pub(crate) static ASSET_CATALOG: OnceLock<AssetCatalog> = OnceLock::new();

static GAME_DATA: OnceLock<GameDataAssets> = OnceLock::new();

pub(crate) fn game_data() -> &'static GameDataAssets {
    GAME_DATA.get().expect("game data not initialized")
}

fn init_game() -> Result<()> {
    log::info!("Init game start");

    let assets_path_config =
        load_toml::<AssetPathConfig>(Path::new("assets.toml")).context("loading assets.toml")?;
    println!("Loaded assets config: {assets_path_config:#?}");
    let catalog = AssetCatalog::new(Arc::new(assets_path_config), AssetResolver);

    ASSET_CATALOG
        .set(catalog)
        .map_err(|_| anyhow!("failed to set global asset catalog"))?;

    let db = DataLoader::new_with_fallback("data", "../assets/data")
        .context("failed to create data loader")?
        .load_all_data()
        .context("failed to load game data")?;
    let db = GAME_DATA.get_or_init(|| db);

    let mut service = PlayerService::load_or_create(GodotUserStorage, &db.models, None)
        .context("failed to initialize progress service")?;
    if dishaster_validation::validate_player_profile(service.profile(), &db.models).is_err() {
        log::warn!("Player profile validation failed, resetting to default profile");
        service.recreate_profile(&db.models, None)?;
    };
    PROGRESS_SERVICE
        .set(Mutex::new(service))
        .map_err(|_| anyhow!("Progress service already initialized"))?;

    log::info!("Progress service initialized");

    dishrupt_l10n_godot::init();
    log::info!("Localization initialized");

    /* setup_preference(); */

    Ok(())
}

/*
fn setup_preference() {
    let pref = local_store().get_preference().expect("preference");

    let mut audio_server = AudioServer::singleton();

    let bus = audio_server.get_bus_index("Music");
    audio_server.set_bus_volume_db(bus, pref.music_volume as f32 / 100.0);
    audio_server.set_bus_mute(bus, pref.music_mute);

    let bus = audio_server.get_bus_index("Sound");
    audio_server.set_bus_volume_db(bus, pref.sound_volume as f32 / 100.0);
    audio_server.set_bus_mute(bus, pref.sound_mute);
}
 */
