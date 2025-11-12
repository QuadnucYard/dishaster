//! Dishrupt Godot audio manager.

#![feature(once_cell_get_mut)]

use std::{cell::OnceCell, collections::HashMap};

use dishrupt_asset::{AssetCatalog, AssetKind, ResourceLocator};
use dishrupt_core::asset::AudioRef;
use godot::{
    classes::{AudioStream, AudioStreamPlayer},
    prelude::*,
};

/// Audio manager for playing sounds and music.
pub struct AudioManager {
    audio_root: Gd<Node>,

    sound_players: HashMap<AudioRef, Gd<AudioStreamPlayer>>,
    music_player: OnceCell<Gd<AudioStreamPlayer>>,

    music_volume: f32,
    sound_volume: f32,

    catalog: AssetCatalog,
}

impl AudioManager {
    const MUSIC_BUS: &str = "Music";
    const SOUND_BUS: &str = "Sound";

    /// Create a new audio manager.
    pub fn new(audio_root: Gd<Node>, catalog: AssetCatalog) -> AudioManager {
        Self {
            audio_root,

            sound_players: Default::default(),
            music_player: Default::default(),

            music_volume: 0.0,
            sound_volume: 0.0,

            catalog,
        }
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
    }

    /// Play a sound effect.
    pub fn play_sound(&mut self, sound: &AudioRef) {
        let sound_player = self.sound_players.entry(sound.clone()).or_insert_with(|| {
            let stream = load_sound(&self.catalog, sound);
            let mut player = AudioStreamPlayer::new_alloc();
            player.set_bus(Self::SOUND_BUS);
            player.set_stream(&stream);
            player.set_max_polyphony(3);

            self.audio_root.clone().add_child(&player);

            player
        });
        sound_player.play();
    }

    /// Play background music.
    pub fn play_music(&mut self, music: &AudioRef) {
        let stream = load_music(&self.catalog, music);
        let player = self.music_player.get_mut_or_init(|| {
            let mut player = AudioStreamPlayer::new_alloc();
            player.set_bus(Self::MUSIC_BUS);
            self.audio_root.clone().add_child(&player);
            player
        });

        player.set_stream(&stream);
        player.play();
    }

    /// Stop the background music.
    pub fn stop_music(&mut self) {
        if let Some(player) = self.music_player.get_mut() {
            player.stop();
        }
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
