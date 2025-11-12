//! Utilities for binding framework types to Godot types.

use bevy_math::prelude::*;
use godot::{
    classes::Node,
    obj::{Gd, Inherits},
};

/// Trait for converting from Godot types to framework types.
pub trait FromGodot<T> {
    /// Convert from a Godot type to a framework type.
    fn from_godot(value: T) -> Self;
}

/// Trait for converting from framework types to Godot types.
pub trait IntoGodot<T> {
    /// Convert from a framework type to a Godot type.
    fn into_godot(self) -> T;
}

/// Trait for converting from Godot types to framework simulation types.
pub trait IntoSim<T> {
    /// Convert from a Godot type to a framework simulation type.
    fn into_sim(self) -> T;
}

/// Trait for binding Godot objects to framework types.
pub trait BindGodot<T: Inherits<Node>> {
    /// Create a new framework type from a Godot object.
    fn new(gd: Gd<T>) -> Self;
}

impl<G, S> IntoSim<S> for G
where
    S: FromGodot<G>,
{
    fn into_sim(self) -> S {
        S::from_godot(self)
    }
}

impl FromGodot<godot::builtin::Vector2> for Vec2 {
    fn from_godot(value: godot::builtin::Vector2) -> Self {
        Self::new(value.x, value.y)
    }
}

impl IntoGodot<godot::builtin::Vector2> for Vec2 {
    fn into_godot(self) -> godot::builtin::Vector2 {
        godot::builtin::Vector2::new(self.x, self.y)
    }
}

impl FromGodot<godot::builtin::Vector3> for Vec3 {
    fn from_godot(value: godot::builtin::Vector3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl IntoGodot<godot::builtin::Vector3> for Vec3 {
    fn into_godot(self) -> godot::builtin::Vector3 {
        godot::builtin::Vector3::new(self.x, self.y, self.z)
    }
}

impl FromGodot<godot::builtin::Vector2i> for IVec2 {
    fn from_godot(value: godot::builtin::Vector2i) -> Self {
        Self::new(value.x, value.y)
    }
}

impl IntoGodot<godot::builtin::Vector2i> for IVec2 {
    fn into_godot(self) -> godot::builtin::Vector2i {
        godot::builtin::Vector2i::new(self.x, self.y)
    }
}

impl FromGodot<godot::builtin::Vector3i> for IVec3 {
    fn from_godot(value: godot::builtin::Vector3i) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl IntoGodot<godot::builtin::Vector3i> for IVec3 {
    fn into_godot(self) -> godot::builtin::Vector3i {
        godot::builtin::Vector3i::new(self.x, self.y, self.z)
    }
}
/*
impl FromGodot<godot::builtin::Color> for tdcore_math::Color {
    fn from_godot(value: godot::builtin::Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl IntoGodot<godot::builtin::Color> for tdcore_math::Color {
    fn into_godot(self) -> godot::builtin::Color {
        godot::builtin::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}
 */
