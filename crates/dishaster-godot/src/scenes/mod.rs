pub mod proc;

mod game;
mod start;

use std::sync::Arc;

use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_godot_scene::*;
use dishrupt_godot_utils::BindGodot;
pub use game::GameScene;
use godot::{
    classes::{Node, PackedScene},
    global::godot_error,
    tools::load,
};
pub use start::StartScene;

pub struct DefaultSceneLoader {
    catalog: Arc<AssetCatalog>,
}

impl DefaultSceneLoader {
    pub fn new(catalog: Arc<AssetCatalog>) -> Self {
        Self { catalog }
    }
}

impl SceneLoader for DefaultSceneLoader {
    fn load(&self, id: SceneId) -> Box<dyn Scene> {
        match id {
            StartScene::ID => Box::new(load_scene_as::<StartScene>("start", &self.catalog)),
            GameScene::ID => Box::new(load_scene_as::<GameScene>("game", &self.catalog)),
            _ => unreachable!(),
        }
    }
}

fn load_scene_as<T>(path: &str, catalog: &AssetCatalog) -> T
where
    T: BindGodot<Node>,
{
    let Ok(ResourceLocator::Uri(uri)) = catalog.resolve(AssetKind::Scene, path) else {
        godot_error!("Failed to resolve scene asset: {}", path);
        panic!("Failed to resolve scene asset: {}", path);
    };

    T::new(load::<PackedScene>(&uri).instantiate_as())
}
