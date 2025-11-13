use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_core::asset::SpriteRef;
use godot::{classes::Texture2D, prelude::*};

/// Load the texture for the given sprite reference.
pub fn load_texture_sync(sprite: &SpriteRef, catalog: &AssetCatalog) -> Gd<Texture2D> {
    let Ok(ResourceLocator::Uri(uri)) = catalog.resolve(AssetKind::Texture, sprite.path()) else {
        panic!("failed to resolve texture: {}", sprite.path());
    };
    load(&uri)
}
