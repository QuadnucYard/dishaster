use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct StartMenuGui {
    #[child("%Start")]
    start_btn: ButtonA,
    #[child("%Credits")]
    credits_btn: ButtonA,
    #[child("%Quit")]
    quit_btn: ButtonA,

    // NOTE: when the button is PRESSED, the audio is MUTED
    #[child("%MusicToggle")]
    music_toggle: ButtonA,
    #[child("%SoundToggle")]
    sound_toggle: ButtonA,
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
        self.music_toggle.on_toggle.connect(move |pressed| {
            cmd.push_req(AppRequest::ToggleMusic(pressed));
        });

        let cmd = commands.clone();
        self.sound_toggle.on_toggle.connect(move |pressed| {
            cmd.push_req(AppRequest::ToggleSound(pressed));
        });
    }
}

impl StartMenuGui {
    /// Update the toggle button states from current settings
    pub fn update_from_preferences(&mut self, music_mute: bool, sound_mute: bool) {
        self.music_toggle.set_pressed(music_mute);
        self.sound_toggle.set_pressed(sound_mute);
    }
}
