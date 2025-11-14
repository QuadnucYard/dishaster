# dishrupt-godot-scene

Scene management system for Godot-based games with stack-based architecture, lifecycle management, and customizable transitions.

## Features

- **Stack-based scene management** - Push/pop scenes with automatic history tracking
- **Scene caching** - Optionally cache scenes for instant reactivation
- **Pluggable transitions** - Use different transition effects per scene change (fade, slide, etc.)
- **Procedure system** - Async-like operations for complex scene orchestration
- **Lifecycle hooks** - `ready()`, `enter()`, `leave()`, `process()`, etc.
- **Post-load callbacks** - Initialize scenes with specific data after loading

## Quick Start

```rust
use dishrupt_godot_scene::*;

// 1. Implement Scene trait
struct MenuScene { /* ... */ }
impl Scene for MenuScene { /* ... */ }

// 2. Implement SceneLoader
struct MyLoader;
impl SceneLoader for MyLoader {
    fn load(&self, id: SceneId) -> Box<dyn Scene> { /* ... */ }
}

// 3. Create manager
let mut manager = SceneManager::new(scene_root, MyLoader);

// 4. Process each frame
manager.process(&mut scene_ctx);
```

## Scene Changes with Transitions

Transitions are provided per scene change:

```rust
// In a procedure
impl SceneProcedure for EnterGameProcedure {
    fn process(&mut self, ctx: &mut SceneProcedureContext) -> SceneProcedurePoll {
        // Create transition for this specific scene change
        let trans = Box::new(FadeTransition::new(ctx.scene_root.clone()));

        ctx.scene_stack.change_push_scene(
            ctx.base,
            GameScene::ID,
            Some(trans),  // Transition stored in pending change
        );

        SceneProcedurePoll::Ready
    }
}
```

## Custom Transitions

Implement `SceneTransition` trait for custom effects:

```rust
struct SlideTransition { /* ... */ }

impl SceneTransition for SlideTransition {
    fn transition_out(&mut self, duration: Option<f32>) -> f32 { /* slide out */ }
    fn transition_in(&mut self, duration: Option<f32>) -> f32 { /* slide in */ }
    fn is_transitioning(&self) -> bool { /* check state */ }
    fn process(&mut self) { /* update animation */ }
}

// Use different transitions for different changes
let slide = Box::new(SlideTransition::new(ctx.scene_root.clone()));
ctx.scene_stack.change_push_scene(ctx.base, scene_id, Some(slide));
```

## Architecture

See inline documentation (`cargo doc --open`) for detailed API reference and examples.
