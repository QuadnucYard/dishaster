use std::sync::{Arc, LazyLock, Mutex};

use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_core::asset::PrefabRef;
use godot::{classes::CanvasLayer, prelude::*};

type EffectQueue = Vec<(PrefabRef, Option<Vector2>)>;

// We have to use a global queue because we cannot access EffectOverlay from arbitrary scenes.
static PENDING_EFFECTS: LazyLock<Mutex<EffectQueue>> = LazyLock::new(Default::default);

pub fn pend_effect(prefab: PrefabRef, position: Option<Vector2>) {
    if let Ok(mut guard) = PENDING_EFFECTS.lock() {
        guard.push((prefab, position));
    }
}

pub struct EffectOverlay {
    root: Gd<CanvasLayer>,

    catalog: Arc<AssetCatalog>,
}

impl EffectOverlay {
    pub fn new(mut root: Gd<CanvasLayer>, catalog: Arc<AssetCatalog>) -> Self {
        root.set_layer(10);
        Self { root, catalog }
    }

    pub fn process(&mut self) {
        let Ok(mut guard) = PENDING_EFFECTS.lock() else {
            return;
        };
        for (prefab, position) in guard.drain(..) {
            if let Some(pos) = position {
                self.spawn(&prefab, pos);
            } else {
                self.spawn_at_mouse(&prefab);
            }
        }
    }

    pub fn spawn(&mut self, prefab: &PrefabRef, position: Vector2) {
        let Ok(ResourceLocator::Uri(uri)) = self.catalog.resolve(AssetKind::Prefab, prefab.path())
        else {
            godot_warn!("Failed to resolve prefab: {}", prefab.path());
            return;
        };
        let Ok(scene) = try_load::<PackedScene>(&uri) else {
            godot_warn!("Failed to load prefab scene: {}", uri);
            return;
        };
        let Some(mut instance) = scene.try_instantiate_as::<Node2D>() else {
            godot_warn!("Failed to instantiate prefab scene: {}", uri);
            return;
        };
        instance.set_position(position);
        self.root.add_child(&instance);
    }

    pub fn spawn_at_mouse(&mut self, prefab: &PrefabRef) {
        // Get mouse position from viewport
        let mouse_pos = self
            .root
            .get_viewport()
            .map(|v| v.get_mouse_position())
            .unwrap_or(Vector2::ZERO);
        self.spawn(prefab, mouse_pos);
    }
}
