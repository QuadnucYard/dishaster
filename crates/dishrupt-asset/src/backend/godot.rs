//! Godot backend for the asset system.

use godot::{classes::ResourceLoader, prelude::*};
use godot_binary_resource::BinaryResource;

use crate::{
    ResourceLocator,
    backend::{DataBackend, LoadError},
};

/// Godot resource backend for loading assets via Godot's resource system.
pub struct GodotResourceBackend;

impl DataBackend for GodotResourceBackend {
    fn exists(&self, locator: &ResourceLocator) -> Result<bool, LoadError> {
        let uri = uri_loc(locator)?;
        Ok(ResourceLoader::singleton().exists(uri))
    }

    /// NOTE: This function should be used after [`GodotResourceBackend::list_dir`], where the directories have "/" appended.
    fn is_file(&self, locator: &ResourceLocator) -> Result<bool, LoadError> {
        let uri = uri_loc(locator)?;
        Ok(!uri.ends_with('/'))
    }

    /// NOTE: This function should be used after [`GodotResourceBackend::list_dir`], where the directories have "/" appended.
    fn is_dir(&self, locator: &ResourceLocator) -> Result<bool, LoadError> {
        let uri = uri_loc(locator)?;
        Ok(uri.ends_with('/'))
    }

    fn list_dir(&self, locator: &ResourceLocator) -> Result<Vec<ResourceLocator>, LoadError> {
        let uri = uri_loc(locator)?;
        let items = ResourceLoader::singleton()
            .list_directory(uri)
            .as_slice()
            .iter()
            .map(|path| {
                // Reconstruct full URI
                let item_uri = if uri.ends_with('/') {
                    format!("{}{}", uri, path)
                } else {
                    format!("{}/{}", uri, path)
                };
                ResourceLocator::Uri(item_uri)
            })
            .collect();
        Ok(items)
    }

    fn read_bytes(&self, locator: &ResourceLocator) -> Result<Vec<u8>, LoadError> {
        let uri = uri_loc(locator)?;
        let res = godot::tools::try_load::<BinaryResource>(uri)
            .map_err(|_| LoadError::NotFound(ResourceLocator::Uri(uri.to_string())))?;
        Ok(res.bind().data().to_vec())
    }
}

fn uri_loc(locator: &ResourceLocator) -> Result<&str, LoadError> {
    let ResourceLocator::Uri(uri) = locator else {
        return Err(LoadError::UnsupportedLocation(locator.clone()));
    };
    Ok(uri)
}
