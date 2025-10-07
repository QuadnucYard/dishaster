use std::{
    cell::OnceCell,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use dishaster_core::models::GameModelRegistry;
use dishaster_data::DataLoader;
use dishaster_godot_ui::register_guis;
use dishaster_persistence::ProgressService;
use dishrupt_godot::{audio::AudioManager, ext::NodeExt, input::listener::InputListener};
use dishrupt_godot_scene::{SceneContext, SceneManager};
use dishrupt_godot_ui::GuiManager;
use godot::{
    classes::{CanvasLayer, ProjectSettings},
    prelude::*,
};
use rand::random;

use crate::scenes::{DefaultSceneLoader, proc::*};

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
    // l10n_server: LocalizationServer,
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

        godot_print!("Main loop initialize");

        init_game();

        godot_print!("Init load complete!");

        let mut inner = Inner::new(self.base().clone().upcast());
        inner.ready();
        let _ = self.inner.set(inner);
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
        // input_listener.bind_mut().base_mut().set_process_mode(ProcessMode::ALWAYS);

        Self {
            scene_manager: SceneManager::new(scene_root.upcast(), DefaultSceneLoader),
            gui: GuiManager::new(gui_root.upcast()),
            audio: AudioManager::new(audio_root),
            // l10n_server: Default::default(),
            input_listener,

            late_initialized: false,
        }
    }

    fn ready(&mut self) {
        register_guis(&mut self.gui.registry);
        self.gui.ready();

        self.scene_manager.schedule(StartProcedure);
    }

    fn ready_late(&mut self) {
        println!("collect localized elements...");
        // self.l10n_server.collect(self.ui_master.root.gd());
        // self.l10n_server.update();
    }

    fn process(&mut self, delta: f64) {
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
        self.gui.process();

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

pub static GAME_DATA: OnceLock<Arc<GameModelRegistry>> = OnceLock::new();
pub static PROGRESS_SERVICE: OnceLock<Mutex<ProgressService>> = OnceLock::new();

fn init_game_database(loader: impl FnOnce() -> Arc<GameModelRegistry>) {
    GAME_DATA
        .set(loader())
        .unwrap_or_else(|_| panic!("init game database error"));
}

pub fn progress_service() -> MutexGuard<'static, ProgressService> {
    PROGRESS_SERVICE
        .get()
        .expect("progress service not initialized")
        .lock()
        .expect("progress service poisoned")
}

fn init_game() {
    println!("Init game start");
    init_game_database(|| {
        let db = Arc::new(
            DataLoader::new("../assets/data")
                .unwrap()
                .load_all_data()
                .unwrap(),
        );
        godot_print!("Loaded {} canteens", db.canteens.len());
        db
    });

    let save_dir = {
        let settings = ProjectSettings::singleton();
        let path: GString = settings.globalize_path("user://");
        PathBuf::from(path.to_string())
    };

    let registry = Arc::clone(GAME_DATA.get().expect("game data not initialized"));
    if PROGRESS_SERVICE
        .set(Mutex::new(
            ProgressService::load_or_create(save_dir, registry, None, random())
                .expect("failed to initialize progress service"),
        ))
        .is_err()
    {
        panic!("progress service already initialized");
    }
    /* setup_preference(); */
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
