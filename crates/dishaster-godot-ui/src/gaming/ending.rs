use dishaster_views::EndingView;
use dishrupt_asset::AssetCatalog;
use dishrupt_core::asset::SpriteRef;

use crate::{load::load_texture_sync, prelude::*};

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
}

#[ui_tree_api]
impl UITree for EndingGui {}

impl Gui for EndingGui {
    fn start(&mut self, commands: GuiCommands) {
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
    }
}

impl EndingGui {
    pub fn set_ending_picture(&mut self, picture: &SpriteRef, catalog: &AssetCatalog) {
        let texture = load_texture_sync(picture, catalog);
        self.picture.set_texture(texture);
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
