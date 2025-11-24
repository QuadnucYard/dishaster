use std::cell::OnceCell;

use dishaster_views::EndingView;
use dishrupt_core::asset::SpriteRef;

use crate::prelude::*;

/// Ending screen showing game conclusion
#[derive(UITree)]
#[ui_tree]
pub struct EndingGui {
    #[child("%TitleLabel")]
    title_label: LabelA,

    #[child("%DescriptionLabel")]
    desc_label: LabelA,

    #[child("%Picture")]
    picture: TextureRectA,

    #[child("%ContinueButton")]
    continue_btn: ButtonA,

    #[child("%ExitButton")]
    exit_btn: ButtonA,

    asset_provider: OnceCell<AssetProvider>,
}

#[ui_tree_api]
impl UITree for EndingGui {}

impl Gui for EndingGui {
    fn start(&mut self, commands: GuiCommands, provider: AssetProvider) {
        // Continue button - triggers decision roll (only for GoodReputation ending)
        let cmd = commands.clone();
        self.continue_btn.on_click.connect(move || {
            cmd.push_req(GameRequest::ContinueFromEnding);
        });

        // Exit button - returns to main menu
        let cmd = commands.clone();
        self.exit_btn.on_click.connect(move || {
            cmd.push_req(AppRequest::ExitLevel);
            cmd.push_req(GameRequest::ClearLevel);
        });

        let _ = self.asset_provider.set(provider);
    }
}

impl EndingGui {
    pub fn set_ending_picture(&mut self, picture: &SpriteRef) {
        let provider = self
            .asset_provider
            .get()
            .expect("EndingGui asset provider not set");

        self.picture.set_texture(provider.get_texture(picture));
    }

    /// Show ending screen with the given view
    pub fn show_ending(&mut self, ending: EndingView) {
        let id = ending.id;
        self.title_label.set_text(&tr!("ending--{}.title", id));
        self.desc_label.set_text(&tr!("ending--{}.desc", id));

        // Good ending: optional, show continue button; others: forced exit, no continue
        self.continue_btn.set_visible(ending.can_continue);

        self.show();
    }
}
