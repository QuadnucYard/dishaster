use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_core::asset::SpriteRef;
use godot::{classes::Texture2D, prelude::*};

#[derive(Clone)]
pub struct AssetProvider {
    catalog: AssetCatalog,
}

impl AssetProvider {
    pub fn new(catalog: AssetCatalog) -> Self {
        Self { catalog }
    }

    pub fn get_texture(&self, texture: &SpriteRef) -> Gd<Texture2D> {
        let Ok(ResourceLocator::Uri(uri)) =
            self.catalog.resolve(AssetKind::Texture, texture.path())
        else {
            panic!("failed to resolve texture: {}", texture.path());
        };
        load(&uri)
    }
}
