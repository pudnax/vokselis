use std::time::Instant;

pub struct FrameCounter {
    frame_count: u32,
    accum_time: f32,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            accum_time: 0.,
        }
    }

    pub fn record(&mut self, current_instant: &mut Instant) -> f32 /* dt */ {
        self.accum_time += current_instant.elapsed().as_secs_f32();
        *current_instant = Instant::now();
        self.frame_count += 1;
        if self.frame_count == 100 {
            println!(
                "Avg frame time {}ms",
                self.accum_time * 1000.0 / self.frame_count as f32
            );
            self.accum_time = 0.0;
            self.frame_count = 0;
        }
        self.accum_time
    }
}
