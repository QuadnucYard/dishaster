//! User-wide settings (not tied to player profile)

use serde::{Deserialize, Serialize};

/// Current version of user settings schema
pub const PREFERENCES_VERSION: u32 = 1;

/// User-wide application settings that persist across all profiles.
/// These are preferences about the application itself, not game progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Settings schema version
    pub schema_version: u32,

    /// Audio settings
    pub audio: AudioPreferences,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: PREFERENCES_VERSION,
            audio: AudioPreferences::default(),
        }
    }
}

/// Audio-related user settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioPreferences {
    /// Whether music is muted (true = muted, false = unmuted)
    pub music_mute: bool,

    /// Whether sound effects are muted (true = muted, false = unmuted)
    pub sound_mute: bool,

    /// Music volume (0.0 to 1.0)
    pub music_volume: f32,

    /// Sound effects volume (0.0 to 1.0)
    pub sound_volume: f32,
}

impl Default for AudioPreferences {
    fn default() -> Self {
        Self {
            music_mute: false,
            sound_mute: false,
            music_volume: 1.0,
            sound_volume: 1.0,
        }
    }
}
