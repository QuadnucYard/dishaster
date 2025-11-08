use std::path::Path;

use anyhow::{Result, anyhow};
use dishrupt_persistence::{PersistentStorage, Persister};
use godot::classes::{
    FileAccess, class_macros::private::virtuals::Os::PackedArray, file_access::ModeFlags,
};

pub struct GodotUserStorage;

impl PersistentStorage for GodotUserStorage {
    fn load_or_create_with<T, P: Persister<T>>(
        &mut self,
        path: &str,
        init: impl FnOnce() -> T,
    ) -> Result<T> {
        let file_path = Path::new("user://").join(path);
        let path_str = file_path.to_str().unwrap();
        if !FileAccess::file_exists(path_str) {
            let value = init();
            self.save_with::<T, P>(path, &value)?;
            return Ok(value);
        }

        let bytes = FileAccess::get_file_as_bytes(path_str);
        P::load_bytes_slice(bytes.as_slice())
    }

    fn save_with<T, P: Persister<T>>(&mut self, path: &str, data: &T) -> Result<()> {
        let bytes = P::dump_bytes(data)?;

        let file_path = Path::new("user://").join(path);
        let path_str = file_path.to_str().unwrap();

        let mut file = FileAccess::open(path_str, ModeFlags::WRITE)
            .ok_or_else(|| anyhow!("failed to open file"))?;

        let packed = PackedArray::from(bytes.as_slice());
        if !file.store_buffer(&packed) {
            return Err(anyhow!("failed to write to file"));
        }
        Ok(())
    }
}
