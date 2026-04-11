use std::time::{Duration, Instant};

/// A simple debouncer that tracks the last trigger time
pub struct Debouncer {
    delay: Duration,
    last_trigger: Option<Instant>,
}

impl Debouncer {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: Duration::from_millis(delay_ms),
            last_trigger: None,
        }
    }

    /// Call this when the event occurs
    pub fn trigger(&mut self) {
        self.last_trigger = Some(Instant::now());
    }

    /// Returns true if enough time has passed since the last trigger
    pub fn is_ready(&self) -> bool {
        self.last_trigger
            .map(|t| t.elapsed() >= self.delay)
            .unwrap_or(false)
    }

    /// Reset after handling
    pub fn reset(&mut self) {
        self.last_trigger = None;
    }
}
