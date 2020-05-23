use std::time::Instant;

pub struct Timer {
    start: Instant,
    prev_time: f32
}

impl Timer {

    pub fn new() -> Self {
        let now = Instant::now();

        Timer {
            start: now,
            prev_time: now.elapsed().as_secs() as f32 + now.elapsed().subsec_nanos() as f32 * 0.000000001
        }
    }

    pub fn get_elapsed_time(&self) -> f32 {
        self.start.elapsed().as_secs() as f32 + self.start.elapsed().subsec_nanos() as f32 * 0.000000001
    }

    pub fn get_delta(&mut self) -> f32 {
        let delta = self.get_elapsed_time() - self.prev_time;

        self.prev_time = self.get_elapsed_time();

        delta
    }

}
