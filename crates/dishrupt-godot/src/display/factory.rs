use std::collections::VecDeque;

use dishrupt_core::asset::{PrefabReference, SpriteReference};
use godot::{
    classes::{Node2D, PackedScene, ResourceLoader, Sprite2D, Texture2D},
    prelude::*,
};
use rustc_hash::FxHashMap;

use super::assets;
use crate::display::node::GdNode2D;

struct PooledNode {
    node: GdNode2D,
    entry_time: u32,
}

struct ActiveNode {
    node: GdNode2D,
    prefab: PrefabIndex,
}

type PrefabIndex = u32;

struct FactoryItem {
    /// the prototype of the item
    prefab: Gd<PackedScene>,

    /// inactive nodes in the pool
    pool: VecDeque<PooledNode>,

    /// number of instances
    count: usize,

    /// the id of next instance
    next_id: usize,
}

impl FactoryItem {
    pub fn from_prefab(prefab: Gd<PackedScene>) -> FactoryItem {
        Self {
            prefab,
            pool: Default::default(),
            count: 0,
            next_id: 0,
        }
    }

    pub fn create_instance(&mut self) -> Gd<Node2D> {
        let mut obj = self.prefab.instantiate_as::<Node2D>();
        let name = format!("{}_{}", obj.get_name(), self.next_id);
        obj.set_name(&name);
        self.next_id += 1;
        obj
    }
}

impl Drop for FactoryItem {
    fn drop(&mut self) {
        if !self.pool.is_empty() {
            println!(
                "  drop {} pooled items of `{}`",
                self.pool.len(),
                self.prefab.get_path()
            );
        }
        for mut n in self.pool.drain(..) {
            n.node.destroy();
        }
    }
}

pub struct DisplayFactory {
    res_registry: FxHashMap<PrefabReference, PrefabIndex>,
    items: Vec<FactoryItem>,
    active: Vec<ActiveNode>,
    last_decay_time: u32,
}

impl DisplayFactory {
    /// How often to decay the pools (in ticks)
    const DECAY_INTERVAL: u32 = 60;
    const DECAY_AT_AGE: u32 = 600;

    pub fn new() -> Self {
        Self {
            res_registry: Default::default(),
            items: Default::default(),
            active: Default::default(),
            last_decay_time: 0,
        }
    }

    pub fn init(&mut self) {
        let dummy_prefab = PrefabReference::new("");
        self.res_registry.insert(dummy_prefab.clone(), 0);
        self.items
            .push(FactoryItem::from_prefab(make_empty_prefab()));
    }

    pub fn create(&mut self, prefab: &PrefabReference) -> GdNode2D {
        let item_index = *self.res_registry.entry(prefab.clone()).or_insert_with(|| {
            let next_index = self.items.len() as PrefabIndex;
            self.items
                .push(FactoryItem::from_prefab(load_or_make_prefab_sync(prefab)));
            next_index
        });
        let item = &mut self.items[item_index as usize];
        item.count += 1;
        let node = if let Some(pn) = item.pool.pop_back() {
            // use pooled node
            pn.node
        } else {
            GdNode2D::new(item.create_instance())
        };
        self.active.push(ActiveNode {
            node: GdNode2D::new(node.clone()),
            prefab: item_index,
        });
        node
    }

    pub fn tidy(&mut self, elapsed_time: u32) {
        self.active
            .extract_if(.., |an| !an.node.is_instance_valid())
            .for_each(|an| {
                let item = &mut self.items[an.prefab as usize];
                item.count -= 1;
                item.pool.push_back(PooledNode {
                    node: an.node,
                    entry_time: elapsed_time,
                });
            });
        if elapsed_time - self.last_decay_time >= Self::DECAY_INTERVAL {
            self.decay_pools(elapsed_time);
        }
    }

    fn decay_pools(&mut self, elapsed_time: u32) {
        self.last_decay_time = elapsed_time;
        for item in &mut self.items {
            while let Some(pn) = item.pool.front_mut() {
                if elapsed_time - pn.entry_time < Self::DECAY_AT_AGE {
                    break;
                }
                item.pool.pop_front();
            }
        }
    }

    pub fn flush(&mut self) {
        println!("flush factory");
        self.last_decay_time = 0;
        // clear pooled instances
        println!("  clear {} pooled prefabs", self.items.len());
        self.items.clear();
        // clear active instances
        println!("  clear {} active items", self.active.len());
        self.active.clear();
    }
}

impl Default for DisplayFactory {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_prefab_sync(prefab: &PrefabReference) -> Gd<PackedScene> {
    load(&format!("{}{}.tscn", assets::PREFABS, prefab.path()))
}

/// Load the given prefab from godot resources.
/// If it not exists, assume it is a sprite.
pub fn load_or_make_prefab_sync(prefab: &PrefabReference) -> Gd<PackedScene> {
    // Try loading prefab. We do not use `try_load` to avoid unnecessary debugger error.
    let prefab_path = format!("{}{}.tscn", assets::PREFABS, prefab.path());
    if ResourceLoader::singleton().exists(&prefab_path) {
        return load(&prefab_path);
    }

    let mut scene = PackedScene::new_gd();
    let mut sprite = Sprite2D::new_alloc();
    if let Ok(texture) =
        try_load::<Texture2D>(&format!("{}{}.tres", assets::SPRITES, prefab.path()))
    {
        sprite.set_texture(&texture);
    }
    scene.pack(&sprite);
    scene
}

pub fn make_empty_prefab() -> Gd<PackedScene> {
    let mut scene = PackedScene::new_gd();
    let mut node = Node2D::new_alloc();
    scene.pack(&node);
    node.queue_free();
    scene
}

pub fn load_texture_sync(sprite: &SpriteReference) -> Gd<Texture2D> {
    load(&format!("{}{}.tres", assets::SPRITES, sprite.path()))
}

pub fn try_load_texture_sync(sprite: &SpriteReference) -> Option<Gd<Texture2D>> {
    try_load(&format!("{}{}.tres", assets::SPRITES, sprite.path())).ok()
}
