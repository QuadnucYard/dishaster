use godot::{
    classes::{CanvasLayer, ColorRect, Tween, tween::TweenPauseMode},
    prelude::*,
};

use crate::SceneTransition;

/// Default fade-to-black transition effect.
///
/// Creates a full-screen black overlay that fades in/out to provide smooth
/// transitions between scenes. The overlay runs on a high-layer `CanvasLayer`
/// to ensure it renders on top of all game content.
///
/// # Transition Flow
///
/// 1. **transition_out()**: Fade from transparent to opaque black (screen goes dark)
/// 2. Scene change happens (scene loads/unloads)
/// 3. **transition_in()**: Fade from opaque black to transparent (new scene revealed)
///
/// # Usage
///
/// ```rust,ignore
/// let mut transition = FadeTransition::new(scene_root.clone());
///
/// // Start fade out
/// transition.transition_out(None);
///
/// // In your update loop:
/// transition.process();
///
/// // Check if fade is complete
/// if !transition.is_transitioning() {
///     // Change scene here
///     transition.transition_in(None);
/// }
/// ```
pub struct FadeTransition {
    /// Canvas layer for rendering overlay on top of everything
    canvas_layer: Gd<CanvasLayer>,
    /// Full-screen black rectangle
    fade_rect: Gd<ColorRect>,
    /// Active tween for fade animation
    current_tween: Option<Gd<Tween>>,
    /// Whether a transition is currently in progress
    is_transitioning: bool,
}

impl FadeTransition {
    /// Default transition duration in seconds.
    const DEFAULT_FADE_DURATION: f32 = 0.3;

    /// Create a new fade transition.
    ///
    /// Sets up a full-screen black `ColorRect` on a high-layer `CanvasLayer`
    /// that starts fully transparent.
    ///
    /// # Parameters
    /// - `parent`: Godot node to attach the overlay to (typically scene root)
    pub fn new(mut parent: Gd<Node>) -> Self {
        // Create canvas layer for overlay (renders on top of everything)
        let mut canvas_layer = CanvasLayer::new_alloc();
        canvas_layer.set_name("TransitionOverlay");
        canvas_layer.set_layer(100); // High layer value to render on top

        // Create full-screen black rectangle
        let mut fade_rect = ColorRect::new_alloc();
        fade_rect.set_name("FadeRect");
        fade_rect.set_color(Color::BLACK);
        fade_rect.set_anchor(Side::RIGHT, 1.0);
        fade_rect.set_anchor(Side::BOTTOM, 1.0);
        fade_rect.set_mouse_filter(godot::classes::control::MouseFilter::IGNORE);

        // Start fully transparent
        fade_rect.set_modulate(Color::from_rgba(1.0, 1.0, 1.0, 0.0));

        canvas_layer.add_child(&fade_rect);
        parent.add_child(&canvas_layer);

        Self {
            canvas_layer,
            fade_rect,
            current_tween: None,
            is_transitioning: false,
        }
    }

    /// Kill the current tween if it exists.
    ///
    /// Stops any active fade animation and cleans up the tween object.
    fn kill_current_tween(&mut self) {
        if let Some(mut tween) = self.current_tween.take()
            && tween.is_valid()
        {
            tween.kill();
        }
    }
}

impl SceneTransition for FadeTransition {
    fn transition_out(&mut self, duration: Option<f32>) -> f32 {
        let duration = duration.unwrap_or(Self::DEFAULT_FADE_DURATION);

        self.kill_current_tween();

        // Fade from transparent to opaque black
        let mut tween = self.fade_rect.create_tween();
        tween.set_pause_mode(TweenPauseMode::PROCESS);
        tween.tween_property(
            &self.fade_rect,
            "modulate:a",
            &1.0.to_variant(),
            duration as f64,
        );

        self.current_tween = Some(tween);
        self.is_transitioning = true;

        duration
    }

    fn transition_in(&mut self, duration: Option<f32>) -> f32 {
        let duration = duration.unwrap_or(Self::DEFAULT_FADE_DURATION);

        self.kill_current_tween();

        // Fade from opaque black to transparent
        let mut tween = self.fade_rect.create_tween();
        tween.set_pause_mode(TweenPauseMode::PROCESS);
        tween.tween_property(
            &self.fade_rect,
            "modulate:a",
            &0.0.to_variant(),
            duration as f64,
        );

        self.current_tween = Some(tween);
        self.is_transitioning = true;

        duration
    }

    fn is_transitioning(&self) -> bool {
        self.is_transitioning
    }

    fn process(&mut self) {
        // Check if tween has finished
        if !self.is_transitioning {
            return;
        }

        let tween_finished = self
            .current_tween
            .as_mut()
            .is_none_or(|tween| !tween.is_valid() || !tween.is_running());

        if tween_finished {
            self.is_transitioning = false;
            self.current_tween = None;
        }
    }
}

impl Drop for FadeTransition {
    fn drop(&mut self) {
        self.kill_current_tween();
        if self.canvas_layer.is_instance_valid() {
            self.canvas_layer.queue_free();
        }
    }
}
