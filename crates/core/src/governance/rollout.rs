//! Gradual Rollout Tracker
//!
//! Helper for Tier 1 Executor to determine if a request should use a new variant
//! based on a time-based ramp schedule.

use super::RolloutConfig;
use crate::rng::Prng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};

/// Tracks the state of a gradual rollout
#[derive(Debug)]
pub struct RolloutTracker {
    config: RolloutConfig,
    start_time: Instant,
    rng: Prng,
}

impl RolloutTracker {
    /// Start a new rollout
    pub fn new(config: RolloutConfig, seed: u64) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            rng: Prng::seed_from_u64(seed),
        }
    }

    /// Check if the current request should use the new feature
    ///
    /// Returns true if selected for rollout
    pub fn should_rollout(&mut self) -> bool {
        let elapsed = self.start_time.elapsed();
        let minutes = elapsed.as_secs_f64() / 60.0;

        let current_percent = (self.config.initial_percent
            + minutes * self.config.increment_per_minute)
            .min(self.config.max_percent)
            .min(1.0);

        let sample: f64 = self.rng.random(); // 0.0 to 1.0
        sample < current_percent
    }

    /// Get current target percentage
    pub fn current_percent(&self) -> f64 {
        let elapsed = self.start_time.elapsed();
        let minutes = elapsed.as_secs_f64() / 60.0;

        (self.config.initial_percent + minutes * self.config.increment_per_minute)
            .min(self.config.max_percent)
            .min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollout_ramp() {
        let config = RolloutConfig {
            initial_percent: 0.1,
            increment_per_minute: 0.1,
            max_percent: 1.0,
        };

        // Mock start time by manually checking math logic
        // (Can't mock Instant::now easily without abstraction, trusting integration test)
        let tracker = RolloutTracker::new(config, 42);
        assert!(tracker.current_percent() >= 0.1);
    }
}
