use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_core::asset::PrefabRef;
use godot::{classes::CanvasLayer, prelude::*};

pub struct GlobalEffects {
    pending: Vec<(PrefabRef, Option<Vector2>)>,
}

impl GlobalEffects {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn pend(&mut self, prefab: PrefabRef, position: Option<Vector2>) {
        self.pending.push((prefab, position));
    }

    pub fn process(&mut self, overlay: &mut EffectOverlay) {
        for (prefab, position) in self.pending.drain(..) {
            if let Some(pos) = position {
                overlay.spawn(&prefab, pos);
            } else {
                overlay.spawn_at_mouse(&prefab);
            }
        }
    }
}

pub struct EffectOverlay {
    root: Gd<CanvasLayer>,

    catalog: AssetCatalog,
}

impl EffectOverlay {
    pub fn new(mut root: Gd<CanvasLayer>, catalog: AssetCatalog) -> Self {
        root.set_layer(10);
        Self { root, catalog }
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
