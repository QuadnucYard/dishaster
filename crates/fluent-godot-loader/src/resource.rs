use godot::{
    classes::{IResourceFormatLoader, ResourceFormatLoader, file_access::ModeFlags},
    prelude::*,
};

/// A Godot resource type for fluent assets.
#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct FluentResource {
    base: Base<Resource>,

    text: GString,
}

impl FluentResource {
    /// The string name of this resource type.
    pub const RES_TYPE: &'static str = "FluentResource";

    /// Gets the text content of this resource.
    pub fn text(&self) -> &GString {
        &self.text
    }
}

/// Resource loader for fluent assets.
#[derive(GodotClass)]
#[class(init, tool, base=ResourceFormatLoader)]
pub struct FluentFormatLoader {
    base: Base<ResourceFormatLoader>,
}

#[godot_api]
impl IResourceFormatLoader for FluentFormatLoader {
    // All file extensions you want to be redirected to your loader
    // should be added here.
    fn get_recognized_extensions(&self) -> PackedStringArray {
        ["ftl".to_godot()].into()
    }

    // All resource types that this loader handles.
    fn handles_type(&self, ty: StringName) -> bool {
        ty == FluentResource::RES_TYPE
    }

    // The stringified name of your resource should be returned.
    fn get_resource_type(&self, path: GString) -> GString {
        // The extension arg always comes with a `.` in Godot, so don't forget it ;)
        if path.get_extension().to_lower() == ".ftl" {
            FluentResource::RES_TYPE.into()
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
        let Ok(mut file) = GFile::open(&path, ModeFlags::READ) else {
            return godot::global::Error::ERR_CANT_OPEN.to_variant();
        };
        let mut buf = String::with_capacity(file.length() as usize);
        if file.read_to_string(&mut buf).is_err() {
            return godot::global::Error::ERR_FILE_CANT_READ.to_variant();
        }

        let mut res = FluentResource::new_gd();
        res.bind_mut().text = buf.to_godot();
        res.to_variant()
    }
}
