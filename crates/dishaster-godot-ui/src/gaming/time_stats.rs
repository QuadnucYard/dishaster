use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct TimeStatsGui {
    #[child("%PerfLabel")]
    pub perf_label: LabelA,
    #[child("%TimeLabel")]
    pub time_label: LabelA,
    #[child("%DinerStatsLabel")]
    pub diner_stats_label: LabelA,
    #[child("%TpsSlider")]
    pub tps_slider: SliderA,
    #[child("%TpsValueLabel")]
    pub tps_value_label: LabelA,
    #[child("%DebugSwitch")]
    pub debug_switch: ButtonA,
}

impl TimeStatsGui {
    pub fn update_time(&mut self, sim_tick: u32, sim_time: f64) {
        let total_seconds = sim_time as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let text = format!("Sim: {hours:02}:{minutes:02}:{seconds:02} (tick {sim_tick})");

        self.time_label.set_text(&text);
    }

    pub fn update_perf(&mut self, fps: f32, ups: f32) {
        let text = format!("FPS: {fps:>5.1}  UPS: {ups:>5.1}");

        self.perf_label.set_text(&text);
    }

    pub fn update_diner_stats(
        &mut self,
        current_diners: u32,
        total_visits: u32,
        completed_diners: u32,
        revenue: f32,
        consumption_kg: f32,
    ) {
        let text = format!(
            "Diners: {current_diners} / {total_visits}\nCompleted: {completed_diners}\nConsumed: {consumption_kg:.1}kg\nRevenue: ¥{revenue:.1}"
        );

        self.diner_stats_label.set_text(&text);
    }

    /// Update the displayed ticks-per-second value and keep the slider in sync.
    pub fn set_tps_display(&mut self, tps: f32) {
        let clamped = tps.max(1.0);
        self.tps_value_label.set_text(&format!("{clamped:.0}"));
        if (self.tps_slider.get_value() - clamped).abs() > f32::EPSILON {
            self.tps_slider.set_value(clamped);
        }
    }
}

#[ui_tree_api]
impl UITree for TimeStatsGui {}

impl Gui for TimeStatsGui {
    fn start(&mut self, commands: GuiCommands) {
        let cmd = commands.clone();
        self.tps_slider.on_value_change.connect(move |value| {
            cmd.push_req(GameRequest::SetTps(value));
        });

        let cmd = commands.clone();
        self.debug_switch.on_toggle.connect(move |pressed| {
            cmd.push_req(GameRequest::SetDebugMode(pressed));
        });
    }
}
