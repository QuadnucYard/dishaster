//! Tests for persistence of user progress.

use std::sync::Arc;

use anyhow::Result;
use dishaster_persistence::UserDataService;
use dishrupt_persistence::FsStorage;
use tempfile::tempdir;

#[test]
fn new_user_receives_first_day_level() -> Result<()> {
    let dir = tempdir()?;
    let service = UserDataService::new(Arc::new(FsStorage::new(dir.path().to_path_buf()).unwrap()));

    assert!(service.profiles.load().unwrap().level_progress.is_none());
    Ok(())
}
