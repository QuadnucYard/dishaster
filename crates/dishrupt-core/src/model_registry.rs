//! Type-safe model registry for data storage and retrieval

use std::marker::PhantomData;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Unique identifier for game object models
///
/// Provides a type-safe string-based identifier for referencing
/// model definitions. Used for serialization, configuration files,
/// and human-readable model references.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(EcoString);

impl ModelId {
    /// Create a new model identifier
    pub fn new(name: impl Into<EcoString>) -> Self {
        Self(name.into())
    }

    pub fn to_string(self) -> EcoString {
        self.0
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trait for models that have a unique identifier
pub trait HasId {
    /// Get the unique identifier for this model
    fn id(&self) -> &ModelId;
}

/// Type-safe handle for referencing stored models
///
/// Provides efficient access to models without storing the data directly.
/// The generic type parameter ensures compile-time type safety when
/// accessing model data from registries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelHandle<T>(usize, PhantomData<T>);

impl<T> ModelHandle<T> {
    /// Create a new handle pointing to the given index
    ///
    /// This is private to ensure only ModelRegistry can create handles,
    /// maintaining the integrity of handle-to-model relationships.
    fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }
}

impl<T: Clone> Copy for ModelHandle<T> {}

/// Efficient storage and retrieval system for game object models
///
/// Provides string-based lookup and type-safe handle-based access to
/// model definitions. Supports both unique naming and efficient runtime
/// access patterns required for simulation performance.
#[derive(Resource, Debug)]
pub struct ModelRegistry<T> {
    /// Dense array of model instances for efficient iteration and access
    models: Vec<T>,
    /// HashMap for O(1) name-to-index lookup during configuration loading
    name_to_handle: FxHashMap<ModelId, usize>,
}

impl<T> ModelRegistry<T> {
    /// Create a new empty model registry
    pub fn new() -> Self {
        Default::default()
    }

    /// Store a model with the given identifier and return a type-safe handle
    ///
    /// If a model with the same name already exists, returns the existing handle
    /// without storing a duplicate. This ensures name uniqueness within each registry.
    ///
    /// # Returns
    /// Handle that can be used to efficiently retrieve the model later
    pub fn intern(&mut self, id: ModelId, model: T) -> ModelHandle<T> {
        if let Some(&idx) = self.name_to_handle.get(&id) {
            return ModelHandle::new(idx);
        }
        let idx = self.models.len();
        self.models.push(model);
        self.name_to_handle.insert(id, idx);
        ModelHandle::new(idx)
    }

    /// Retrieve a model using its type-safe handle
    ///
    /// # Panics
    /// Panics if the handle is invalid (should never happen with proper usage)
    pub fn get(&self, handle: ModelHandle<T>) -> &T {
        &self.models[handle.0]
    }

    /// Find a model handle by its string identifier
    pub fn get_by_id(&self, id: &ModelId) -> Option<&T> {
        self.name_to_handle.get(id).map(|idx| &self.models[*idx])
    }

    /// Find a model by its string identifier
    pub fn get_handle_by_id(&self, id: &ModelId) -> Option<ModelHandle<T>> {
        self.name_to_handle.get(id).copied().map(ModelHandle::new)
    }

    /// Get the first model in the registry, if any exist
    ///
    /// Useful for scenarios where only one model of a type is expected,
    /// or for accessing a default/fallback model.
    pub fn first(&self) -> Option<&T> {
        self.models.first()
    }

    /// Get the number of models currently stored in the registry
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

impl<T> Default for ModelRegistry<T> {
    fn default() -> Self {
        Self {
            models: Default::default(),
            name_to_handle: Default::default(),
        }
    }
}
