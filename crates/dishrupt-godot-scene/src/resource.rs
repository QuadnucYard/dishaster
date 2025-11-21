use std::{
    any::{Any, TypeId},
    collections::HashMap,
    panic,
};

use variadics_please::all_tuples;

/// A type-map for scene-local resources.
///
/// This structure stores arbitrary "resources" keyed by their `TypeId` and
/// boxed as `dyn Any`. It acts as a small, typed container for scene-specific
/// data that systems can insert and retrieve by concrete type. The storage is
/// intentionally minimal and panic-on-miss when looking up unknown types, as
/// this mirrors typical ECS/resource access patterns.
#[derive(Default)]
pub struct SceneResources {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl SceneResources {
    /// Create an empty `SceneResources` map.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert `resource` into the map, replacing any existing value of the
    /// same concrete type `T`.
    ///
    /// The resource is stored by type only. Use the concrete Rust type when
    /// retrieving (e.g., `get::<MyResource>()`).
    pub fn insert<T: 'static>(&mut self, resource: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Return a shared reference to a resource of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if a resource of the requested type has not been inserted.
    pub fn get<T: 'static>(&self) -> &T {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
            .unwrap_or_else(|| panic!("Resource {} not found", std::any::type_name::<T>()))
    }

    /// Return a mutable reference to a resource of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if a resource of the requested type has not been inserted.
    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T>())
            .unwrap_or_else(|| panic!("Resource {} not found", std::any::type_name::<T>()))
    }

    /// Return a tuple of shared references to multiple resources.
    ///
    /// The requested set of types is described by the `SceneResourceState`
    /// implementation selected by the caller (for example, `self.get_many::<(A, B)>()`).
    /// This mirrors the common pattern of batch resource access.
    pub fn get_many<'a, T: SceneResourceState<'a>>(&'a self) -> T::Output {
        T::get(self)
    }

    /// Return a tuple of mutable references to multiple resources.
    ///
    /// This method ensures disjoint mutable access where possible (via
    /// `get_disjoint_mut`) and will panic if a resource is missing.
    pub fn get_many_mut<'a, T: SceneResourceState<'a>>(&'a mut self) -> T::OutputMut {
        T::get_mut(self)
    }
}

/// Trait for extracting multiple typed references from `SceneResources`.
///
/// Implementations for tuples of up to 16 types are generated below. The
/// trait provides both shared and mutable accessors (`get` / `get_mut`) that
/// return tuples of references to the requested types.
pub trait SceneResourceState<'a> {
    type Output;
    type OutputMut;

    fn get(resources: &'a SceneResources) -> Self::Output;
    fn get_mut(resources: &'a mut SceneResources) -> Self::OutputMut;
}

macro_rules! impl_scene_resource_state {
    ($(($T:ident, $t:ident)),*) => {
        impl<'a, $($T: 'static),*> SceneResourceState<'a> for ($($T),* ,) {
            type Output = ($( &'a $T ),* ,);
            type OutputMut = ($( &'a mut $T ),* ,);

            fn get(resources: &'a SceneResources) -> Self::Output {
                (
                    $(
                        resources
                            .map
                            .get(&TypeId::of::<$T>())
                            .and_then(|t| t.downcast_ref())
                            .unwrap_or_else(|| panic!("Resource {} not found", std::any::type_name::<$T>()))
                    ),*
                    ,
                )
            }

            fn get_mut(resources: &'a mut SceneResources) -> Self::OutputMut {
                let [ $($t),* ] = resources
                    .map
                    .get_disjoint_mut([ $( &TypeId::of::<$T>() ),* ]);
                (
                    $(
                        $t
                            .and_then(|t| t.downcast_mut())
                            .unwrap_or_else(|| panic!("Resource {} not found", std::any::type_name::<$T>()))
                    ),*
                    ,
                )
            }
        }

    };
}

all_tuples!(impl_scene_resource_state, 1, 16, T, t);
