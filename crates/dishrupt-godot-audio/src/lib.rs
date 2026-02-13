//! Dishrupt Godot audio manager.

use std::collections::HashMap;

use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_core::asset::AudioRef;
use godot::{
    classes::{AudioServer, AudioStream, AudioStreamPlayer, Tween, tween::TweenPauseMode},
    prelude::*,
};

/// State of the background music system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MusicState {
    /// No music playing
    Idle,
    /// Music is playing normally
    Playing,
    /// Music is fading out (transitioning to another track)
    FadingOut,
    /// Music is fading in (new track starting)
    FadingIn,
    /// Music is paused
    Paused,
}

/// Background music manager with cross-fade support
struct BgmManager {
    /// Current music player (source being faded out or currently playing)
    current_player: Option<Gd<AudioStreamPlayer>>,
    /// Player that was paused (for resume)
    paused_player: Option<Gd<AudioStreamPlayer>>,
    /// Tween for smooth volume transitions
    tween: Option<Gd<Tween>>,

    /// Current state of the music system
    state: MusicState,
    /// Current track being played
    current_track: Option<AudioRef>,
    /// Target volume (0-1 linear scale) - used when resuming or adjusting volume
    target_volume: f32,
    /// Cross-fade duration in seconds
    fade_duration: f32,
}

impl BgmManager {
    /// Default cross-fade duration
    const DEFAULT_FADE_DURATION: f32 = 0.5;

    fn new() -> Self {
        Self {
            current_player: None,
            paused_player: None,
            tween: None,
            state: MusicState::Idle,
            current_track: None,
            target_volume: 1.0,
            fade_duration: Self::DEFAULT_FADE_DURATION,
        }
    }

    /// Check if currently playing a specific track
    fn is_playing(&self, track: &AudioRef) -> bool {
        self.current_track.as_ref() == Some(track) && self.state == MusicState::Playing
    }
}

/// Audio manager for playing sounds and music.
pub struct AudioManager {
    audio_root: Gd<Node>,

    sound_players: HashMap<AudioRef, Gd<AudioStreamPlayer>>,
    bgm: BgmManager,

    music_mute: bool,
    sound_mute: bool,
    music_volume: f32,
    sound_volume: f32,

    catalog: AssetCatalog,
}

impl AudioManager {
    const MUSIC_BUS: &str = "Music";
    const SOUND_BUS: &str = "Sound";

    /// Create a new audio manager.
    pub fn new(audio_root: Gd<Node>, catalog: AssetCatalog) -> AudioManager {
        let mut bgm = BgmManager::new();
        bgm.target_volume = 1.0; // Default to full volume

        Self {
            audio_root,

            sound_players: Default::default(),
            bgm,

            music_mute: false,
            sound_mute: false,
            music_volume: 1.0, // Default music volume
            sound_volume: 1.0, // Default sound volume

            catalog,
        }
    }

    /// Set whether music is mute.
    pub fn set_music_mute(&mut self, mute: bool) {
        self.music_mute = mute;

        // TODO: for volume, we may also need to modify bus
        let mut audio_server = AudioServer::singleton();

        let bus = audio_server.get_bus_index(Self::MUSIC_BUS);
        audio_server.set_bus_mute(bus, mute);
    }

    /// Set whether sound effects are mute.
    pub fn set_sound_mute(&mut self, mute: bool) {
        self.sound_mute = mute;

        // TODO: for volume, we may also need to modify bus
        let mut audio_server = AudioServer::singleton();

        let bus = audio_server.get_bus_index(Self::SOUND_BUS);
        audio_server.set_bus_mute(bus, mute);
    }

    /// Get the current sound volume.
    pub fn get_sound_volume(&self) -> f32 {
        self.sound_volume
    }

    /// Set the sound volume.
    pub fn set_sound_volume(&mut self, volume: f32) {
        self.sound_volume = volume;
    }

    /// Get the current music volume.
    pub fn get_music_volume(&self) -> f32 {
        self.music_volume
    }

    /// Set the music volume.
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume;
        self.bgm.target_volume = volume;

        // Update current player volume if playing
        if self.bgm.state == MusicState::Playing
            && let Some(player) = &mut self.bgm.current_player
        {
            player.set_volume_db(linear_to_db(volume));
        }
    }

    /// Play a sound effect.
    pub fn play_sound(&mut self, sound: &AudioRef) {
        let sound_player = self.sound_players.entry(sound.clone()).or_insert_with(|| {
            let stream = load_sound(&self.catalog, sound);
            let mut player = AudioStreamPlayer::new_alloc();
            player.set_name(sound.path().as_str());
            player.set_bus(Self::SOUND_BUS);
            player.set_stream(&stream);
            player.set_max_polyphony(3);

            self.audio_root.add_child(&player);

            player
        });
        sound_player.play();
    }

    /// Play background music with cross-fade transition.
    ///
    /// If a different track is currently playing, it will fade out while the new track fades in.
    /// If the same track is already playing, this does nothing.
    pub fn play_music(&mut self, music: &AudioRef) {
        self.start_music_crossfade(music, self.bgm.fade_duration)
    }

    /// Play background music with loop enabled (default for BGM).
    pub fn play_music_loop(&mut self, music: &AudioRef) {
        self.start_music_crossfade(music, self.bgm.fade_duration)
    }

    /// Play background music with custom fade duration and cross-fade from current track.
    pub fn play_music_crossfade(&mut self, music: &AudioRef, fade_duration: f32) {
        self.start_music_crossfade(music, fade_duration)
    }

    /// Pause current background music.
    pub fn pause_music(&mut self) {
        // Clear any previously paused player
        if let Some(mut player) = self.bgm.paused_player.take() {
            godot_print!("Clear previously paused player on pause");
            player.queue_free();
        }

        // Move current player to paused
        if let Some(mut player) = self.bgm.current_player.take() {
            godot_print!("Pausing current music player");
            player.set_stream_paused(true);
            self.bgm.paused_player = Some(player);

            self.bgm.state = MusicState::Paused;
        }
    }

    /// Resume the paused background music.
    ///
    /// Continues playing from where it left off.
    pub fn resume_music(&mut self) {
        godot_print!("Resuming paused music player");

        if let Some(mut paused_player) = self.bgm.paused_player.take() {
            paused_player.set_stream_paused(false);
            let current_player = self.bgm.current_player.take();
            self.crossfade_with(paused_player, current_player, self.bgm.fade_duration);
        }
    }

    /// Start cross-fade transition to new music track.
    fn start_music_crossfade(&mut self, music: &AudioRef, fade_duration: f32) {
        // Skip if already playing this track
        if self.bgm.is_playing(music) {
            return;
        }

        let stream = load_music(&self.catalog, music);

        // Create new player for the next track
        let mut next_player = AudioStreamPlayer::new_alloc();
        next_player.set_name(music.path().as_str());
        next_player.set_bus(Self::MUSIC_BUS);
        next_player.set_stream(&stream);
        next_player.set_autoplay(false);
        next_player.set_volume_db(linear_to_db(0.0)); // Start silent

        self.audio_root.add_child(&next_player);

        let old_player = self.bgm.current_player.take();
        self.crossfade_with(next_player, old_player, fade_duration);

        // Promote next player to current
        self.bgm.current_track = Some(music.clone());

        godot_print!("Started music cross-fade to track: {:?}", music);
    }

    fn crossfade_with(
        &mut self,
        mut next_player: Gd<AudioStreamPlayer>,
        old_player: Option<Gd<AudioStreamPlayer>>,
        fade_duration: f32,
    ) {
        // Start playing the new track
        next_player.play();

        // Handle cross-fade based on current state
        if let Some(old_player) = old_player {
            // Fade out old player, fade in new player
            self.bgm.state = MusicState::FadingOut;

            // Create tween for cross-fade
            let mut tween = self.audio_root.create_tween();
            tween.set_pause_mode(TweenPauseMode::PROCESS);
            tween.set_parallel(); // Run both tweens simultaneously

            // Fade out old player
            tween.tween_property(
                &old_player,
                "volume_db",
                &linear_to_db(0.0).to_variant(),
                fade_duration as f64,
            );

            // Fade in new player
            tween.tween_property(
                &next_player,
                "volume_db",
                &linear_to_db(self.bgm.target_volume).to_variant(),
                fade_duration as f64,
            );

            // When fade completes, free the old player
            tween
                .tween_callback(&old_player.callable("queue_free"))
                .set_delay(fade_duration as f64);

            self.bgm.tween = Some(tween);
        } else {
            // No previous music, just fade in
            self.bgm.state = MusicState::FadingIn;

            let mut tween = self.audio_root.create_tween();
            tween.set_pause_mode(TweenPauseMode::PROCESS);
            tween.tween_property(
                &next_player,
                "volume_db",
                &linear_to_db(self.bgm.target_volume).to_variant(),
                fade_duration as f64,
            );

            self.bgm.tween = Some(tween);
        }

        self.bgm.current_player = Some(next_player);
        self.bgm.state = MusicState::Playing;
    }

    /// Stop the background music.
    pub fn stop_music(&mut self) {
        if let Some(mut player) = self.bgm.current_player.take() {
            player.queue_free();
        }

        self.bgm.current_track = None;
        self.bgm.state = MusicState::Idle;
    }

    /// Stop all sound effects.
    pub fn stop_all_sounds(&mut self) {
        for player in self.sound_players.values_mut() {
            player.stop();
        }
    }
}

fn load_sound(catalog: &AssetCatalog, sound: &AudioRef) -> Gd<AudioStream> {
    let ResourceLocator::Uri(uri) = catalog
        .resolve(AssetKind::Sound, sound.path())
        .unwrap_or_else(|e| panic!("failed to resolve sound: {sound}: {e}"))
    else {
        panic!("expected URI for sound asset");
    };
    load(&uri)
}

fn load_music(catalog: &AssetCatalog, music: &AudioRef) -> Gd<AudioStream> {
    let ResourceLocator::Uri(uri) = catalog
        .resolve(AssetKind::Music, music.path())
        .unwrap_or_else(|e| panic!("failed to resolve music: {music}: {e}"))
    else {
        panic!("expected URI for sound asset");
    };
    load(&uri)
}

/// Convert linear volume (0-1) to decibels for Godot AudioStreamPlayer
fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0 {
        -80.0 // Godot's minimum volume
    } else {
        20.0 * linear.log10()
    }
}
