//! A Godot resource loader and resource type for binary assets.

use godot::{
    classes::{
        IResourceFormatLoader, ResourceFormatLoader, ResourceLoader, file_access::ModeFlags,
    },
    prelude::*,
};

// The definition of the singleton with all your loader/savers as members,
// to keep the object references for destruction later.
/// Godot singleton that registers the binary asset loader.
#[derive(GodotClass)]
#[class(base=Object, tool)]
pub struct BinaryAssetSingleton {
    base: Base<Object>,
    loader: Gd<BinaryAssetLoader>,
}

#[godot_api]
impl IObject for BinaryAssetSingleton {
    fn init(base: Base<Object>) -> Self {
        let loader = BinaryAssetLoader::new_gd();

        // Register the loader and saver in Godot.
        //
        // If you want your default extension to be the one defined by your loader,
        // set the `at_front` parameter to true. Otherwise you can also remove the
        // builder. Godot currently doesn't provide a way to completely deactivate
        // the built-in loaders.
        //
        // WARNING: The built-in loaders won't work if you have _pure Rust state_.
        ResourceLoader::singleton().add_resource_format_loader(&loader);

        Self { base, loader }
    }
}

// Unregister the loader and saver when the extension is unloaded.
impl Drop for BinaryAssetSingleton {
    fn drop(&mut self) {
        ResourceLoader::singleton().remove_resource_format_loader(&self.loader);
    }
}

/// Resource loader for binary assets.
#[derive(GodotClass)]
#[class(init, tool, base=ResourceFormatLoader)]
pub struct BinaryAssetLoader {
    base: Base<ResourceFormatLoader>,
}

#[godot_api]
impl IResourceFormatLoader for BinaryAssetLoader {
    // All file extensions you want to be redirected to your loader
    // should be added here.
    fn get_recognized_extensions(&self) -> PackedStringArray {
        ["bin".to_godot(), "ron".to_godot(), "toml".to_godot()].into()
    }

    // All resource types that this loader handles.
    fn handles_type(&self, ty: StringName) -> bool {
        ty == "BinaryResource".into()
    }

    // The stringified name of your resource should be returned.
    fn get_resource_type(&self, path: GString) -> GString {
        // The extension arg always comes with a `.` in Godot, so don't forget it ;)
        if path.get_extension().to_lower() == ".bin".into()
            || path.get_extension().to_lower() == ".ron".into()
            || path.get_extension().to_lower() == ".toml".into()
        {
            "BinaryResource".into()
        } else {
            // In case of not handling the given resource, this function must
            // return an empty string.
            GString::new()
        }
    }

    // The actual loading and parsing of your data.
    fn load(
        &self,

        // The path that should be openend to load the resource.
        path: GString,

        // If the resource was part of a import step you can access the original file
        // with this. Otherwise this path is equal to the normal path.
        _original_path: GString,

        // This parameter is true when the resource is loaded with
        // load_threaded_request().
        // Internal implementations in Godot also ignore this parameter.
        _use_sub_threads: bool,

        // If you want to provide custom caching this parameter is the CacheMode enum.
        // You can look into the ResourceLoader docs to learn about the values.
        // When calling the default load() method, cache_mode is CacheMode::REUSE.
        _cache_mode: i32,
    ) -> Variant {
        use std::io::Read;
        println!("Loading binary asset at path: {}", path);
        let path = if path.begins_with("res://") {
            path
        } else {
            format!("res://{}", path).to_godot()
        };
        let Ok(mut file) = GFile::open(&path, ModeFlags::READ) else {
            return godot::global::Error::ERR_CANT_OPEN.to_variant();
        };
        let mut buf = Vec::new();
        let _ = file.read_to_end(&mut buf);

        let mut res = BinaryResource::new_gd();
        res.bind_mut().data = buf.into();
        res.to_variant()
    }
}

/// A Godot resource type for binary data.
#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct BinaryResource {
    base: Base<Resource>,

    data: PackedByteArray,
}

impl BinaryResource {
    /// Get the binary data stored in this resource.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}
