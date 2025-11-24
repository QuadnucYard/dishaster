use std::cell::OnceCell;

use dishrupt_core::{asset::SpriteRef, prelude::EcoString};

use crate::prelude::*;

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

    asset_provider: OnceCell<AssetProvider>,
}

#[ui_tree_api]
impl UITree for EndingGalleryGui {}

impl Gui for EndingGalleryGui {
    fn start(&mut self, commands: GuiCommands, provider: AssetProvider) {
        let cmd = commands.clone();
        self.illustration.on_click.connect(move || {
            cmd.push_req(AppRequest::BackToMenu);
        });

        let _ = self.asset_provider.set(provider);
    }
}

impl EndingGalleryGui {
    pub fn show_ending(&mut self, view: EndingGalleryView) {
        let provider = self
            .asset_provider
            .get()
            .expect("EndingGalleryGui asset provider not set");

        self.illustration
            .set_texture_normal(&provider.get_texture(&view.illustration));

        self.title_label.set_text(&tr!("ending--{}.title", view.id));

        self.show();
    }
}
