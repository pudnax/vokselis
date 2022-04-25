use std::time::Instant;

pub struct FrameCounter {
    pub frame_count: u32,
    accum_time: f32,
    last_inst: Instant,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time_delta(&self) -> f32 {
        self.accum_time * 1000.0 / self.frame_count as f32
    }

    pub fn record(&mut self) -> f32 /* dt */ {
        self.accum_time += self.last_inst.elapsed().as_secs_f32();
        self.last_inst = Instant::now();

        self.frame_count += 1;
        if self.frame_count == 100 {
            println!("Avg frame time {}ms", self.time_delta());
            self.accum_time = 0.0;
            self.frame_count = 0;
        }
        self.accum_time
    }
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            frame_count: 0,
            accum_time: 0.,
            last_inst: Instant::now(),
        }
    }
}
