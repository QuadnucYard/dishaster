use std::{cell::OnceCell, sync::Arc};

use anyhow::{Context, Result};
use dishaster_data::DataLoader;
use dishaster_godot_ui::register_guis;
use dishrupt_asset::{AssetPathConfig, AssetResolver};
use dishrupt_godot_input::listener::InputListener;
use dishrupt_godot_ui::UiRoot;
use dishrupt_l10n_godot::LocalizationManager;
use dishrupt_persistence::GodotUserStorage;
use godot::{classes::CanvasLayer, prelude::*};

use crate::{
    effect::{EffectOverlay, GlobalEffects},
    panic::{get_panic_message, has_panic_occurred, init_backtrace_handle},
    panic_overlay::PanicOverlay,
    prelude::*,
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
    gui_root: UiRoot,

    scene_manager: SceneManager,
    resources: SceneResources,

    l10n: LocalizationManager,
    input_listener: Gd<InputListener>,
    effect_overlay: EffectOverlay,
    panic_overlay: PanicOverlay,

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
        godot_print!(
            "Index: {}",
            godot::classes::ResourceLoader::singleton().exists("res://assets.toml")
        );
        godot_print!(
            "ftl: {}",
            godot::classes::ResourceLoader::singleton().exists("res://locales/zh-CN/credits.ftl")
        );

        match std::panic::catch_unwind(init_game) {
            Ok(Ok(services)) => {
                log::info!("Init game completed successfully");

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
    fn new(mut root: Gd<Node>, services: GameServices) -> Inner {
        let scene_root = root.get_or_add_node_as::<Node2D>("SceneRoot");
        let ui_root = root.get_or_add_node_as::<CanvasLayer>("UIRoot");
        let audio_root = root.get_or_add_node_as("AudioRoot");

        let scene_manager = SceneManager::new(
            scene_root.upcast(),
            DefaultSceneLoader::new(services.catalog.clone()),
        );
        let audio = AudioManager::new(audio_root, services.catalog.clone());

        let l10n = Default::default();
        let input_listener = root.get_or_add_node_of_type::<InputListener>();

        let effect_overlay = EffectOverlay::new(
            root.get_or_add_node_as("EffectOverlay"),
            services.catalog.clone(),
        );
        let panic_overlay = PanicOverlay::new(root.get_node_as("%PanicOverlay"));

        let mut scene_res = SceneResources::new();
        scene_res.insert(GuiRegistry::new());
        scene_res.insert(GuiCommands::new());
        scene_res.insert(audio);
        scene_res.insert(GlobalEffects::new());
        scene_res.insert(services.catalog);
        scene_res.insert(services.data);
        scene_res.insert(services.user_service);

        Self {
            gui_root: UiRoot::new(ui_root.upcast()),

            scene_manager,
            resources: scene_res,
            l10n,

            input_listener,
            effect_overlay,
            panic_overlay,

            late_initialized: false,
            panic_displayed: false,
        }
    }

    fn ready(&mut self) {
        {
            let (gui, gui_cmds, catalog) = self
                .resources
                .get_many_mut::<(GuiRegistry, GuiCommands, AssetCatalog)>();

            register_guis(gui, catalog);
            gui.mount(&mut self.gui_root, gui_cmds);
        }

        self.apply_preferences();
        self.init_profile();

        self.scene_manager.schedule(StartProcedure);
    }

    fn ready_late(&mut self) {
        godot_print!("collect localized elements...");
        self.l10n.collect(self.gui_root.gd());
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
                res: &mut self.resources,
                proc: None,
            };

            // process scene events
            self.scene_manager.process(ctx);
        }

        // process UI
        {
            let (gui, gui_cmds) = self.resources.get_many_mut::<(GuiRegistry, GuiCommands)>();
            gui.process(delta, gui_cmds);
        }

        // process active scene
        {
            let ctx = &mut SceneContext {
                res: &mut self.resources,
                proc: None,
            };
            self.scene_manager.inspect_active_scene_mut(|scene| {
                scene.process(ctx, delta);
            });
            if let Some(proc) = ctx.proc.take() {
                self.scene_manager.schedule1(proc);
            }
        }

        {
            let effects = self.resources.get_mut::<GlobalEffects>();
            effects.process(&mut self.effect_overlay);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if has_panic_occurred() {
            return;
        }
        init_backtrace_handle();

        self.scene_manager.inspect_active_scene_mut(|scene| {
            let ctx = &mut SceneContext {
                res: &mut self.resources,
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
        let user_svc = self.resources.get::<UserDataService>();

        let prefs = user_svc.prefs.load().expect("failed to load preferences");
        let audio_prefs = &prefs.audio;

        let audio = self.resources.get_mut::<AudioManager>();
        audio.set_music_mute(audio_prefs.music_mute);
        audio.set_sound_mute(audio_prefs.sound_mute);
        audio.set_music_volume(audio_prefs.music_volume);
        audio.set_sound_volume(audio_prefs.sound_volume);
    }

    fn init_profile(&mut self) {
        let user_svc = self.resources.get::<UserDataService>();

        let profile_svc = &user_svc.profiles;

        match profile_svc.load() {
            Ok(profile) => {
                let data = self.resources.get::<GameDataAssets>();
                if dishaster_validation::validate_player_profile(&profile, &data.models).is_err() {
                    log::warn!("Player profile validation failed, resetting to default profile");
                    if let Err(e) = profile_svc.create() {
                        log::error!("Failed to recreate profiles: {e}");
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to load profile: {e}. Recreating default profile");
                if let Err(e) = profile_svc.create() {
                    log::error!("Failed to recreate profiles: {e}");
                }
            }
        }
    }
}

fn init_game() -> Result<GameServices> {
    log::info!("Init game start");

    // let data_backend = GodotResourceBackend;
    let (catalog, data) = load_data().context("loading game data")?;

    let user_service = UserDataService::new(Arc::new(GodotUserStorage));

    dishrupt_l10n_godot::init();
    log::info!("Localization initialized");

    Ok(GameServices {
        catalog,
        data,
        user_service,
    })
}

/// Load asset catalog and game data in non-production builds.
#[cfg(not(feature = "production"))]
fn load_data() -> Result<(AssetCatalog, GameDataAssets)> {
    use std::path::Path;

    use dishaster_data::load_toml;
    use dishrupt_asset::backend::FsBackend;

    let backend = FsBackend::new(Path::new("../assets/data"))?;

    let assets_path_config =
        load_toml::<AssetPathConfig>("assets.toml").context("loading assets.toml")?;
    println!("Loaded assets config: {assets_path_config:#?}");
    let catalog = AssetCatalog::new(Arc::new(assets_path_config), AssetResolver);

    let data = DataLoader::new(catalog.clone(), backend)
        .load_all_data()
        .context("failed to load game data")?;

    Ok((catalog, data))
}

#[cfg(feature = "production")]
fn load_data() -> Result<(AssetCatalog, GameDataAssets)> {
    use dishaster_data::load_toml_with;
    use dishrupt_asset::{AssetKind, ResourceLocator, backend::GodotResourceBackend};

    let backend = GodotResourceBackend;

    let mut assets_path_config = load_toml_with::<AssetPathConfig>(
        &ResourceLocator::Uri("res://assets.toml".into()),
        &backend,
    )
    .context("loading assets.toml")?;
    assets_path_config
        .kinds
        .entry(AssetKind::Data)
        .or_insert_with(Default::default)
        .prefix = "res://data/".into(); // Ensure data assets use correct prefix
    println!("Loaded assets config: {assets_path_config:#?}");
    let catalog = AssetCatalog::new(Arc::new(assets_path_config), AssetResolver);

    let data = DataLoader::new(catalog.clone(), backend)
        .load_all_data()
        .context("failed to load game data")?;

    Ok((catalog, data))
}

struct GameServices {
    catalog: AssetCatalog,
    data: GameDataAssets,
    user_service: UserDataService,
}
