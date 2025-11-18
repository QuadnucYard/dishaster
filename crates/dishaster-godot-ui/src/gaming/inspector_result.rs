use dishaster_views::InspectorResultView;

use crate::prelude::*;

/// GUI for displaying inspector visit result (pass/fail with boosts)
#[derive(UITree)]
#[ui_tree]
pub struct InspectorResultGui {
    #[child("%EffectsLabel")]
    effects_label: RichLabelA,

    #[child("%ConfirmButton")]
    confirm_button: ButtonA,
}

#[ui_tree_api]
impl UITree for InspectorResultGui {}

impl Gui for InspectorResultGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.confirm_button.on_click.connect(move || {
            cmd.hide::<Self>();
        });
    }
}

impl InspectorResultGui {
    /// Set the view data for the inspector result display
    pub fn set_view(&mut self, view: &InspectorResultView) {
        // Display boost values
        self.effects_label.set_text(&tr!(
            "inspector-visit-effects",
            "trust_boost" = view.trust_boost,
            "reputation_boost" = view.reputation_boost,
        ));
    }
}
