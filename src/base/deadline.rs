use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Deadline {
    start: Instant,
    limit: Duration,
}

impl Deadline {
    pub fn new(limit: Duration) -> Self {
        Self {
            start: Instant::now(),
            limit,
        }
    }

    pub fn from_secs_f32(secs: f32) -> Self {
        Self::new(Duration::from_secs_f32(secs))
    }

    pub fn expired(&self) -> bool {
        self.start.elapsed() >= self.limit
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}
