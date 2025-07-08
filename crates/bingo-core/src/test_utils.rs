//! Performance testing utilities

use std::time::Instant;

/// A simple timer to measure elapsed time.
pub struct Timer {
    start_time: Instant,
}

impl Timer {
    /// Creates a new timer and starts it.
    pub fn new() -> Self {
        Self { start_time: Instant::now() }
    }

    /// Returns the elapsed time since the timer was created.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
