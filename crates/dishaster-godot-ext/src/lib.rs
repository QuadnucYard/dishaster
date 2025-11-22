extern crate dishaster_godot;

use godot::prelude::*;

struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {
    fn override_wasm_binary() -> Option<&'static str> {
        Some("dishaster.wasm")
    }
}
