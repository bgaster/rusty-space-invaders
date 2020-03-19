//! Description: 
//! 
//! Really (really) basic timer. Designed for polling synchronosuly.
//! 
//! Copyright © 2020 Benedict Gaster. All rights reserved.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Timer {
    /// duration that time is to run for, when reset
    duration: Duration,
    /// time since last reset
    start: Instant,
}

impl Timer {
    /// create a timer that last for duration
    pub fn new(duration: Duration) -> Self {
        Timer {
            duration,
            // reset timer to now
            start: Instant::now(),
        }
    }

    /// reset time from now
    pub fn reset(&mut self) {
        self.start = Instant::now()
    }

    /// check if time since last reset is great than timer duration, 
    /// return true if the case, otherwise false. will continue to return true, until reset
    pub fn has_expired(&self) -> bool {
        Instant::now() - self.start >= self.duration 
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }
}
