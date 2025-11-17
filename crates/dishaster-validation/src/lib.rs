//! Data validation for game model and profile integrity

mod models;
mod profiles;

use dishrupt_core::model_registry::ModelId;
use thiserror::Error;

pub use self::{models::validate_registry, profiles::validate_player_profile};

/// Validation error types
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    /// Referenced model ID does not exist
    #[error("Missing {model_type} reference: '{id}' (referenced from {context})")]
    MissingReference {
        /// Type of model being referenced
        model_type: &'static str,
        /// Missing model ID
        id: ModelId,
        /// Context where reference was found
        context: String,
    },

    /// Invalid configuration value
    #[error("Invalid {field} in {context}: {reason}")]
    InvalidValue {
        /// Field name with invalid value
        field: &'static str,
        /// Context where invalid value was found
        context: String,
        /// Reason why value is invalid
        reason: String,
    },

    /// Duplicate model ID
    #[error("Duplicate {model_type} ID: '{id}'")]
    DuplicateId {
        /// Type of model with duplicate ID
        model_type: &'static str,
        /// Duplicated model ID
        id: ModelId,
    },

    /// Empty required collection
    #[error("Empty {collection} in {context}")]
    EmptyCollection {
        /// Name of empty collection
        collection: &'static str,
        /// Context where empty collection was found
        context: String,
    },
}

/// Result type for validation operations
pub type ValidationResult<T = ()> = Result<T, Vec<ValidationError>>;

/// Helper to collect validation errors
pub(crate) struct ErrorSink(Vec<ValidationError>);

impl ErrorSink {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn push(&mut self, error: ValidationError) {
        self.0.push(error);
    }

    pub(crate) fn collect(&mut self, result: ValidationResult) {
        if let Err(mut errors) = result {
            self.0.append(&mut errors);
        }
    }

    pub(crate) fn finish(self) -> ValidationResult {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.0)
        }
    }
}
