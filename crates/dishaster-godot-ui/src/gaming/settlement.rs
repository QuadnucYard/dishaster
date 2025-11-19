use dishaster_views::SettlementView;

use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct SettlementGui {
    #[child("%DayHeading")]
    day_heading: LabelA,

    #[child("%StatsLabel")]
    stats_label: RichLabelA,

    #[child("%GuidanceLabel")]
    guidance_label: RichLabelA,

    #[child("%ConfirmButton")]
    confirm_btn: ButtonA,
}

#[ui_tree_api]
impl UITree for SettlementGui {}

impl Gui for SettlementGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.confirm_btn.on_click.connect(move || {
            cmd.push_req(GameRequest::ConfirmSettlement);
        });
    }
}

impl SettlementGui {
    /// Update the settlement display with day statistics and reputation data
    pub fn set_view(&mut self, view: &SettlementView) {
        // Update day heading
        self.day_heading
            .set_text(&tr!("settlement-title", "day" = view.day));

        // Format completion rate
        let completion_rate = if view.total_visits > 0 {
            (view.completed_diners as f32 / view.total_visits as f32) * 100.0
        } else {
            0.0
        };

        // Format reputation change with sign
        let reputation_sign = if view.reputation_delta >= 0.0 {
            "+"
        } else {
            ""
        };

        // Build statistics display with localized labels
        let stats_text = tr!(
            "settlement-stats",
            "total_visits" = view.total_visits,
            "completed_diners" = view.completed_diners,
            "completion_rate" = completion_rate,
            "revenue" = view.revenue,
            "consumption_kg" = view.consumption_kg,
            "avg_serving_time" = view.avg_serving_time,
            "avg_dining_time" = view.avg_dining_time,
            "reputation" = view.reputation,
            "reputation_delta" = format!("{}{:.1}", reputation_sign, view.reputation_delta),
            "fsri" = view.fsri,
            "food_quality" = view.food_quality,
        );

        self.stats_label.set_text(&stats_text);

        // Update guidance text
        self.guidance_label.set_text(&tr!("settlement-guidance"));
    }
}
