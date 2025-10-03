use crate::prelude::*;

#[derive(UITree)]
#[ui_tree]
pub struct TimeStatsGui {
    #[child("%PerfLabel")]
    pub perf_label: LabelA,
    #[child("%TimeLabel")]
    pub time_label: LabelA,
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
}

#[ui_tree_api]
impl UITree for TimeStatsGui {}

impl Gui for TimeStatsGui {
    fn start(&mut self, _cmd: GuiCommands) {}
}
