use std::cell::OnceCell;

use dishaster_views::ManagementIncidentView;

use crate::prelude::*;

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

    asset_provider: OnceCell<AssetProvider>,
}

#[ui_tree_api]
impl UITree for ManageIncidentGui {}

impl Gui for ManageIncidentGui {
    fn start(&mut self, commands: GuiCommands, provider: AssetProvider) {
        let cmd = commands.clone();
        self.confirm_btn.on_click.connect(move || {
            cmd.hide::<Self>();
        });

        let _ = self.asset_provider.set(provider);
    }
}

impl ManageIncidentGui {
    /// Display active event intro
    pub fn set_view(&mut self, view: &ManagementIncidentView) {
        let provider = self
            .asset_provider
            .get()
            .expect("ManageIncidentGui asset provider not set");

        self.icon.set_texture(provider.get_texture(&view.icon));
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
