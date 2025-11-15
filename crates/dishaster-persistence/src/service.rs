//! Service layer for managing user data persistence.

mod pref;
mod profile;

use std::sync::Arc;

use dishrupt_persistence::PersistentStorage;

pub use crate::service::{pref::PreferencesService, profile::ProfileService};

/// High-level service for managing all user data.
///
/// Aggregates preferences and player profile services with shared storage backend.
pub struct UserDataService {
    /// User preferences.
    pub prefs: Arc<PreferencesService>,
    /// Player profile.
    pub profiles: Arc<ProfileService>,
}

impl UserDataService {
    /// Create a new user data service with the specified storage backend.
    pub fn new(store: Arc<dyn PersistentStorage>) -> Self {
        Self {
            prefs: Arc::new(PreferencesService::new(store.clone())),
            profiles: Arc::new(ProfileService::new(store)),
        }
    }
}
