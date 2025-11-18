use dishaster_views::ManagementIncidentView;
use dishrupt_asset::AssetCatalog;

use crate::{load::load_texture_sync, prelude::*};

#[derive(UITree)]
#[ui_tree]
pub struct ManageIncidentGui {
    #[child("%IncidentIcon")]
    icon: TextureRectA,
    #[child("%TitleLabel")]
    title_label: LabelA,
    #[child("%DescLabel")]
    desc_label: RichLabelA,
    #[child("%FlavorLabel")]
    flavor_label: RichLabelA,
    #[child("%EffectsLabel")]
    effects_label: RichLabelA,
    #[child("%ConfirmButton")]
    confirm_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for ManageIncidentGui {}

impl Gui for ManageIncidentGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.confirm_btn.on_click.connect(move || {
            cmd.hide::<Self>();
        });
    }
}

impl ManageIncidentGui {
    /// Display active event intro
    pub fn set_view(&mut self, view: &ManagementIncidentView, catalog: &AssetCatalog) {
        self.icon
            .set_texture(load_texture_sync(&view.icon, catalog));
        self.title_label
            .set_text(&tr!("mgmt--{}.title", view.model_id));
        self.desc_label
            .set_text(&tr!("mgmt--{}.desc", view.model_id));
        self.flavor_label
            .set_text(&tr!("mgmt--{}.flavor", view.model_id));
        self.effects_label
            .set_text(&tr!("mgmt--{}.effects", view.model_id));
    }
}
