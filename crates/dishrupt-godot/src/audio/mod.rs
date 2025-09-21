use std::{cell::OnceCell, collections::HashMap};

use dishrupt_core::asset::SoundReference;
use godot::{
    classes::{AudioStream, AudioStreamPlayer},
    prelude::*,
};

use crate::display::assets;

pub struct AudioManager {
    audio_root: Gd<Node>,

    sound_players: HashMap<SoundReference, Gd<AudioStreamPlayer>>,
    music_player: OnceCell<Gd<AudioStreamPlayer>>,

    music_volume: f32,
    sound_volume: f32,
}

impl AudioManager {
    const MUSIC_BUS: &str = "Music";
    const SOUND_BUS: &str = "Sound";

    pub fn new(audio_root: Gd<Node>) -> AudioManager {
        Self {
            audio_root,

            sound_players: Default::default(),
            music_player: Default::default(),

            music_volume: 0.0,
            sound_volume: 0.0,
        }
    }

    pub fn get_sound_volume(&self) -> f32 {
        self.sound_volume
    }

    pub fn set_sound_volume(&mut self, volume: f32) {
        self.sound_volume = volume;
    }

    pub fn get_music_volume(&self) -> f32 {
        self.music_volume
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume;
    }

    pub fn play_sound(&mut self, sound: &SoundReference) {
        let sound_player = self.sound_players.entry(sound.clone()).or_insert_with(|| {
            let stream = load_sound(sound);
            let mut player = AudioStreamPlayer::new_alloc();
            player.set_bus(Self::SOUND_BUS);
            player.set_stream(&stream);
            player.set_max_polyphony(3);

            self.audio_root.clone().add_child(&player);

            player
        });
        sound_player.play();
    }

    pub fn play_music(&mut self, music: &SoundReference) {
        let player = self.music_player.get_mut_or_init(|| {
            let mut player = AudioStreamPlayer::new_alloc();
            player.set_bus(Self::MUSIC_BUS);
            self.audio_root.clone().add_child(&player);
            player
        });

        let stream = load_music(music);
        player.set_stream(&stream);
        player.play();
    }

    pub fn stop_music(&mut self) {
        if let Some(player) = self.music_player.get_mut() {
            player.stop();
        }
    }

    pub fn stop_all_sounds(&mut self) {
        for player in self.sound_players.values_mut() {
            player.stop();
        }
    }
}

fn load_sound(sound: &SoundReference) -> Gd<AudioStream> {
    load(&format!("{}{}", assets::SOUNDS, sound.path()))
}

fn load_music(sound: &SoundReference) -> Gd<AudioStream> {
    load(&format!("{}{}", assets::MUSICS, sound.path()))
}
