//! Data models for game credits.
//!
//! These are not used in gameplay, but for dependency, we place them here.

use super::prelude::*;

/// Complete credits data
#[derive(Default, Deserialize)]
pub struct CreditsData {
    /// List of credit sections
    pub sections: Vec<CreditSection>,
}

/// A single section in the credits
#[derive(Deserialize)]
pub struct CreditSection {
    /// L10n key for the section title (e.g., "game-design")
    pub title: String,
    /// List of names/entries in this section
    pub entries: Vec<String>,
}
