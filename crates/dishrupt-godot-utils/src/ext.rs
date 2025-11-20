//! Utilities for extending Godot types with additional functionality.

use godot::{
    classes::Node,
    obj::{Bounds, bounds, cap},
    prelude::*,
};

/// Extension trait for Godot Object.
pub trait ObjectExt {
    /// Clear all connections for a given signal.
    fn clear_connections(&mut self, signal: &str);
}

impl ObjectExt for godot::classes::Object {
    fn clear_connections(&mut self, signal: &str) {
        for conn in self.get_signal_connection_list(signal).iter_shared() {
            self.disconnect(
                &conn.at("signal").to::<Signal>().name(),
                &conn.at("callable").to(),
            );
        }
    }
}

/// Extension trait for Godot Node.
pub trait NodeExt {
    /// Add a child at a specific index.
    fn add_child_at<T>(&mut self, child: &Gd<T>, index: i32)
    where
        T: GodotClass + Inherits<Node>;

    /// Get the first child of a specific type.
    fn get_child_of_type<T>(&self) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>;

    /// Get the first ancestor of a specific type.
    fn get_ancestor_of_type<T>(&self) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>;

    /// Get the first descendant of a specific type.
    fn get_descendant_of_type<T>(&self) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>;

    /// Get or add a child node of a specific type.
    fn get_or_add_node_of_type<T>(&mut self) -> Gd<T>
    where
        T: GodotClass + Inherits<Node> + cap::GodotDefault + Bounds<Memory = bounds::MemManual>;

    /// Add a child node of a specific type with a given name.
    fn add_child_as<T>(&mut self, name: &str) -> Gd<T>
    where
        T: GodotClass + Inherits<Node> + cap::GodotDefault + Bounds<Memory = bounds::MemManual>;

    /// Get or add a child node of a specific type with a given name.
    fn get_or_add_node_as<T>(&mut self, name: &str) -> Gd<T>
    where
        T: GodotClass + Inherits<Node> + cap::GodotDefault + Bounds<Memory = bounds::MemManual>;
}

impl NodeExt for Node {
    fn add_child_at<T>(&mut self, child: &Gd<T>, index: i32)
    where
        T: GodotClass + Inherits<Node>,
    {
        self.add_child(child);
        self.move_child(child, index);
    }

    fn get_child_of_type<T>(&self) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>,
    {
        for ch in self.get_children().iter_shared() {
            if let Ok(casted) = ch.try_cast::<T>() {
                return Some(casted);
            }
        }
        None
    }

    fn get_ancestor_of_type<T>(&self) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>,
    {
        let mut cur = self.get_parent();
        while let Some(p) = cur {
            if let Ok(casted) = p.clone().try_cast::<T>() {
                return Some(casted);
            }
            cur = p.get_parent();
        }
        None
    }

    fn get_descendant_of_type<T>(&self) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>,
    {
        for ch in self.get_children().iter_shared() {
            if let Ok(casted) = ch.clone().try_cast::<T>() {
                return Some(casted);
            }
            if let Some(res) = ch.get_descendant_of_type::<T>() {
                return Some(res);
            }
        }
        None
    }

    fn get_or_add_node_of_type<T>(&mut self) -> Gd<T>
    where
        T: GodotClass + Inherits<Node> + cap::GodotDefault + Bounds<Memory = bounds::MemManual>,
    {
        for ch in self.get_children().iter_shared() {
            if let Ok(casted) = ch.try_cast::<T>() {
                return casted;
            }
        }
        let child = T::new_alloc();
        self.add_child(&child);
        child
    }

    fn add_child_as<T>(&mut self, name: &str) -> Gd<T>
    where
        T: GodotClass + Inherits<Node> + cap::GodotDefault + Bounds<Memory = bounds::MemManual>,
    {
        let mut child = T::new_alloc();
        child.upcast_mut::<Node>().set_name(name);
        self.add_child(&child);
        child
    }

    fn get_or_add_node_as<T>(&mut self, name: &str) -> Gd<T>
    where
        T: GodotClass + Inherits<Node> + cap::GodotDefault + Bounds<Memory = bounds::MemManual>,
    {
        self.try_get_node_as(name).unwrap_or_else(|| {
            let mut child = T::new_alloc();
            // child.set_name(path);
            child.upcast_mut::<Node>().set_name(name);
            self.add_child(&child);
            child
        })
    }
}

/// Extension trait for Godot AnimationPlayer.
pub trait AnimationPlayerExt {
    /// Play an animation by name.
    fn play_by_name(&mut self, name: &str);

    /// Reset the animation player to its initial state.
    fn reset(&mut self) {
        self.play_by_name("RESET");
    }
}

impl AnimationPlayerExt for godot::classes::AnimationPlayer {
    fn play_by_name(&mut self, name: &str) {
        self.play_ex().name(name).done();
    }
}
