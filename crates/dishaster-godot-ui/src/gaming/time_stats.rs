use crate::prelude::*;

#[derive(Debug, Default)]
pub struct TimeStatsD {
    pub fps_estimate: f64,
    pub ups_estimate: f64,
    pub last_sim_time: f64,
    pub last_sim_tick: u64,
}

#[derive(UITree)]
#[ui_tree]
pub struct TimeStatsGui {
    #[child("%StatsHudLabel")]
    pub hud_label: LabelA,
}

impl TimeStatsGui {
    pub fn update(&mut self, stats: &TimeStatsD) {
        let total_seconds = stats.last_sim_time.max(0.0).floor() as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let fps_display = if stats.fps_estimate.is_nan() {
            "--".to_string()
        } else {
            format!("{:.1}", stats.fps_estimate)
        };
        let ups_display = if stats.ups_estimate.is_nan() {
            "--".to_string()
        } else {
            format!("{:.1}", stats.ups_estimate)
        };

        let text = format!(
            "FPS: {:>5}  UPS: {:>5}\nSim: {:02}:{:02}:{:02} (tick {})",
            fps_display, ups_display, hours, minutes, seconds, stats.last_sim_tick,
        );

        self.hud_label.set_text(&text);
    }
}

#[ui_tree_api]
impl UITree for TimeStatsGui {}

impl Gui for TimeStatsGui {
    fn start(&mut self, _cmd: GuiCommands) {}
}
