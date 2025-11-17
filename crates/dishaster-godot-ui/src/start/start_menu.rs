use std::collections::{HashMap, HashSet};

use dishrupt_core::prelude::EcoString;

use crate::prelude::*;

macro_rules! hash_map {
    () => {{
        std::collections::HashMap::default()
    }};

    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map = std::collections::HashMap::default();
        $( map.insert($key, $value); )*
        map
    }}
}

#[derive(UITree)]
#[ui_tree]
pub struct StartMenuGui {
    #[child("%Start")]
    start_btn: ButtonA,
    #[child("%Credits")]
    credits_btn: ButtonA,
    #[child("%Quit")]
    quit_btn: ButtonA,
    #[child("%DeleteProfile")]
    delete_profile_btn: ButtonA,

    // NOTE: when the button is PRESSED, the audio is MUTED
    #[child("%MusicToggle")]
    music_toggle: ButtonA,
    #[child("%SoundToggle")]
    sound_toggle: ButtonA,

    // Ending gallery buttons. Hardcoded IDs for now.
    #[init(hash_map! {
        "good_reputation" => ButtonA::new(root.child("%EndingButtons/GoodReputation")),
        "bad_reputation" => ButtonA::new(root.child("%EndingButtons/BadReputation")),
        "rectification" => ButtonA::new(root.child("%EndingButtons/Rectification")),
    })]
    ending_buttons: HashMap<&'static str, ButtonA>,
}

#[ui_tree_api]
impl UITree for StartMenuGui {}

impl Gui for StartMenuGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.start_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::EnterLevel);
        });

        let cmd = commands.clone();
        self.credits_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::ShowCredits);
        });

        let cmd = commands.clone();
        self.quit_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::Quit);
        });

        let cmd = commands.clone();
        self.delete_profile_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::DeleteProfile);
        });

        let cmd = commands.clone();
        self.music_toggle.on_toggle.connect(move |pressed| {
            cmd.push_req(AppRequest::ToggleMusic(pressed));
        });

        let cmd = commands.clone();
        self.sound_toggle.on_toggle.connect(move |pressed| {
            cmd.push_req(AppRequest::ToggleSound(pressed));
        });

        // Wire up ending button handlers
        for (&ending_id, button) in &self.ending_buttons {
            let cmd = commands.clone();
            button.on_click.connect(move || {
                cmd.push_req(AppRequest::ViewEnding(ending_id.into()));
            });
        }
    }
}

impl StartMenuGui {
    /// Update the toggle button states from current settings
    pub fn update_from_preferences(&mut self, music_mute: bool, sound_mute: bool) {
        self.music_toggle.set_pressed(music_mute);
        self.sound_toggle.set_pressed(sound_mute);
    }

    /// Update ending buttons based on which endings are unlocked
    pub fn update_endings_unlocked(&mut self, achieved_endings: &HashSet<EcoString>) {
        for (&ending_id, button) in &mut self.ending_buttons {
            // For testing, enable all endings
            button.set_enabled(achieved_endings.contains(ending_id));
        }
    }
}
