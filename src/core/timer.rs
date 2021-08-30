use std::time::{Instant, Duration};

pub struct Timer {
    start: Instant,
    prev_time: f32,
    elapsed_time: f32,
    delta_time: f32,
}

impl Timer {

    pub fn new() -> Self {
        let now = Instant::now();

        Timer {
            start: now,
            prev_time: Self::duration_as_seconds(&now.elapsed()),
            elapsed_time: 0.0,
            delta_time: 0.0,
        }
    }

    pub fn elapsed_time(&self) -> f32 {
       self.elapsed_time
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub(crate) fn tick(&mut self) {
        self.elapsed_time = Self::duration_as_seconds(&self.start.elapsed());
        self.delta_time = self.elapsed_time - self.prev_time;

        self.prev_time = self.elapsed_time
    }

    fn duration_as_seconds(duration: &Duration) -> f32 {
        duration.as_nanos() as f32 * 1e-9
    }
}
