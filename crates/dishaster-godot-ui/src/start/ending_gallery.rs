use dishrupt_asset::AssetCatalog;
use dishrupt_core::{asset::SpriteRef, prelude::EcoString};

use crate::{load::load_texture_sync, prelude::*};

pub struct EndingGalleryView {
    pub id: EcoString,
    pub illustration: SpriteRef,
}

/// Ending screen showing game conclusion
#[derive(UITree)]
#[ui_tree]
pub struct EndingGalleryGui {
    #[child("%Illustration")]
    illustration: TextureButtonA,
    #[child("%TitleLabel")]
    title_label: LabelA,
}

#[ui_tree_api]
impl UITree for EndingGalleryGui {}

impl Gui for EndingGalleryGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.illustration.on_click.connect(move || {
            cmd.push_req(AppRequest::BackToMenu);
        });
    }
}

impl EndingGalleryGui {
    pub fn show_ending(&mut self, view: EndingGalleryView, catalog: &AssetCatalog) {
        let texture = load_texture_sync(&view.illustration, catalog);
        self.illustration.set_texture_normal(&texture);

        self.title_label.set_text(&tr!("ending--{}.title", view.id));

        self.show();
    }
}
