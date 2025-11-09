use dishaster_views::{ManagementDecisionView, ManagementDecisionsView};

use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct ManageDecisionGui {
    // #[child("%DayLabel")]
    // day_label: LabelA,
    #[new(root.child_ui("%Options"), root.child_ui("%DecisionOptionTemplate"))]
    options: PooledContainer<DecisionOptionItem>,

    on_select_option: signals2::Signal<(usize,)>,
}

#[ui_tree_api]
impl UITree for ManageDecisionGui {}

impl Gui for ManageDecisionGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.on_select_option.connect(move |index| {
            cmd.push_req(GameRequest::SelectDecision(index));
        });
    }
}

impl ManageDecisionGui {
    pub fn set_view(&mut self, view: &ManagementDecisionsView) {
        // Clear existing cards
        self.options.clear();

        for (i, option) in view.options.iter().enumerate() {
            let item = self.options.get();
            item.set_view(option);

            let on_select_option_handle = self.on_select_option.get_emit_handle();
            item.select_btn.on_click.clear();
            item.select_btn.on_click.connect(move || {
                on_select_option_handle.emit(i);
            });
        }
    }
}

#[derive(UITree)]
#[ui_tree]
struct DecisionOptionItem {
    #[child("%TitleLabel")]
    title_label: LabelA,
    #[child("%DescLabel")]
    desc_label: RichLabelA,
    #[child("%FlavorLabel")]
    flavor_label: RichLabelA,
    #[child("%EffectsLabel")]
    effects_label: RichLabelA,
    #[child("%SelectButton")]
    select_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for DecisionOptionItem {}

impl DecisionOptionItem {
    pub fn set_view(&mut self, view: &ManagementDecisionView) {
        self.title_label
            .set_text(&tr!("mgmt--{}.title", view.model_id));
        self.desc_label
            .set_text(&tr!("mgmt--{}.desc", view.model_id));
        self.flavor_label
            .set_text(&tr!("mgmt--{}.flavor", view.model_id));
        self.effects_label
            .set_text(&tr!("mgmt--{}.effects", view.model_id; view.params.to_fluent()));
    }
}
