#[derive(Debug, Default)]
pub struct PerfTracker {
    frames: u32,
    updates: u32,
    elapsed: f64,

    pub last_fps: f32,
    pub last_ups: f32,
}

impl PerfTracker {
    const SAMPLE_INTERVAL: f64 = 0.5;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick_frame(&mut self) {
        self.frames += 1;
    }

    pub fn tick_update(&mut self) {
        self.updates += 1;
    }

    pub fn tick_updates(&mut self, n: u32) {
        self.updates += n;
    }

    pub fn sample(&mut self, delta: f64) {
        self.elapsed += delta;
        if self.elapsed >= Self::SAMPLE_INTERVAL {
            self.last_fps = self.frames as f32 / self.elapsed as f32;
            self.last_ups = self.updates as f32 / self.elapsed as f32;
            self.frames = 0;
            self.updates = 0;
            self.elapsed = 0.0;
        }
    }
}
