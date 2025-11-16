use crate::prelude::*;

/// Model representing a game ending
#[derive(Debug, Clone, Deserialize)]
pub struct EndingModel {
    /// Illustration associated with the ending
    pub illustration: SpriteRef,
}
