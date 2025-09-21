pub mod proc;

mod game;
mod start;

use dishrupt_godot::{bind::BindGodot, display::assets};
use dishrupt_godot_scene::*;
pub use game::GameScene;
use godot::{
    classes::{Node, PackedScene},
    tools::load,
};
pub use start::StartScene;

pub struct DefaultSceneLoader;

impl SceneLoader for DefaultSceneLoader {
    fn load(&self, id: SceneId) -> Box<dyn Scene> {
        match id {
            StartScene::ID => Box::new(load_scene_as::<StartScene>("start")),
            GameScene::ID => Box::new(load_scene_as::<GameScene>("game")),
            _ => unreachable!(),
        }
    }
}

fn load_scene_as<T>(path: &str) -> T
where
    T: BindGodot<Node>,
{
    T::new(load::<PackedScene>(&format_scene_path(path)).instantiate_as())
}

pub fn format_scene_path(scene_name: &str) -> String {
    format!("{}{scene_name}.tscn", assets::SCENES)
}
