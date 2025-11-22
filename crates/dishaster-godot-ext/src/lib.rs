extern crate dishaster_godot;

use godot::prelude::*;

struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {
    // Register the singleton when the extension is loading.
    #[allow(unused_variables)]
    fn on_level_init(level: InitLevel) {
        #[cfg(feature = "production")]
        if level == InitLevel::Scene {
            use godot::classes::Engine;
            use godot_binary_resource::BinaryAssetSingleton;

            Engine::singleton().register_singleton(
                &BinaryAssetSingleton::class_id().to_string_name(),
                &BinaryAssetSingleton::new_alloc(),
            );
        }
    }

    // Unregister the singleton when the extension is unloaded.
    #[allow(unused_variables)]
    fn on_level_deinit(level: InitLevel) {
        #[cfg(feature = "production")]
        if level == InitLevel::Scene {
            use godot::classes::Engine;
            use godot_binary_resource::BinaryAssetSingleton;

            let mut engine = Engine::singleton();
            let singleton_name = &BinaryAssetSingleton::class_id().to_string_name();
            let my_singleton = engine.get_singleton(singleton_name).unwrap();
            engine.unregister_singleton(singleton_name);
            my_singleton.free();
        }
    }
}
