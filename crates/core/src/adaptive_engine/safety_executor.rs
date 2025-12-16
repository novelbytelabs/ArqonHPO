//! Safety executor with guardrails
//!
//! Enforces bounds, rate limits, max deltas, and rollback capability.
//! This is the Tier 1 component that prevents unsafe parameter updates.

use std::collections::HashMap;
use crate::config::Domain;

/// Guardrails configuration
#[derive(Debug, Clone)]
pub struct Guardrails {
    /// Maximum absolute change per step for any parameter
    pub max_delta_per_step: f64,
    /// Maximum updates per second (rate limiting)
    pub max_updates_per_second: f64,
    /// Minimum interval between updates (microseconds)
    pub min_interval_us: u64,
}

impl Default for Guardrails {
    fn default() -> Self {
        Self {
            max_delta_per_step: 0.1,        // 10% max change per step
            max_updates_per_second: 10.0,    // 10 updates/sec max
            min_interval_us: 100_000,        // 100ms minimum interval
        }
    }
}

/// Violation types that prevent an update
#[derive(Debug, Clone, PartialEq)]
pub enum Violation {
    /// Delta exceeds maximum allowed change
    DeltaTooLarge { param: String, delta: f64, max: f64 },
    /// Update rate limit exceeded
    RateLimitExceeded { updates_per_sec: f64, max: f64 },
    /// Parameter would go out of bounds
    OutOfBounds { param: String, value: f64, min: f64, max: f64 },
    /// Unknown parameter not in allowlist
    UnknownParameter { param: String },
}

/// Safety executor for validating and applying parameter updates
#[derive(Debug)]
pub struct SafetyExecutor {
    guardrails: Guardrails,
    bounds: HashMap<String, Domain>,
    baseline: Option<HashMap<String, f64>>,
    last_update_us: u64,
    updates_in_window: u32,
    window_start_us: u64,
}

impl SafetyExecutor {
    /// Create a new safety executor
    pub fn new(guardrails: Guardrails, bounds: HashMap<String, Domain>) -> Self {
        Self {
            guardrails,
            bounds,
            baseline: None,
            last_update_us: 0,
            updates_in_window: 0,
            window_start_us: 0,
        }
    }
    
    /// Validate a delta before applying
    pub fn validate_delta(
        &self,
        current: &HashMap<String, f64>,
        delta: &HashMap<String, f64>,
    ) -> Result<(), Violation> {
        for (param, d) in delta {
            // Check parameter is in allowlist
            let domain = self.bounds.get(param).ok_or_else(|| {
                Violation::UnknownParameter { param: param.clone() }
            })?;
            
            // Check delta magnitude
            if d.abs() > self.guardrails.max_delta_per_step {
                return Err(Violation::DeltaTooLarge {
                    param: param.clone(),
                    delta: *d,
                    max: self.guardrails.max_delta_per_step,
                });
            }
            
            // Check resulting value would be in bounds
            if let Some(current_val) = current.get(param) {
                let new_val = current_val + d;
                if new_val < domain.min || new_val > domain.max {
                    return Err(Violation::OutOfBounds {
                        param: param.clone(),
                        value: new_val,
                        min: domain.min,
                        max: domain.max,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Clamp all parameters to their bounds
    pub fn clamp_to_bounds(&self, params: &mut HashMap<String, f64>) {
        for (param, value) in params.iter_mut() {
            if let Some(domain) = self.bounds.get(param) {
                *value = value.clamp(domain.min, domain.max);
            }
        }
    }
    
    /// Set baseline for rollback
    pub fn set_baseline(&mut self, params: HashMap<String, f64>) {
        self.baseline = Some(params);
    }
    
    /// Get baseline for rollback
    pub fn baseline(&self) -> Option<HashMap<String, f64>> {
        self.baseline.clone()
    }
    
    /// Check if an update would violate rate limits
    pub fn check_rate_limit(&self, current_time_us: u64) -> Result<(), Violation> {
        // Simple check: minimum interval between updates
        if current_time_us < self.last_update_us + self.guardrails.min_interval_us {
            let elapsed_us = current_time_us.saturating_sub(self.window_start_us);
            let elapsed_sec = elapsed_us as f64 / 1_000_000.0;
            let rate = if elapsed_sec > 0.0 {
                self.updates_in_window as f64 / elapsed_sec
            } else {
                f64::INFINITY
            };
            
            return Err(Violation::RateLimitExceeded {
                updates_per_sec: rate,
                max: self.guardrails.max_updates_per_second,
            });
        }
        
        Ok(())
    }
    
    /// Record that an update was applied
    pub fn record_update(&mut self, current_time_us: u64) {
        // Reset window every second
        if current_time_us > self.window_start_us + 1_000_000 {
            self.window_start_us = current_time_us;
            self.updates_in_window = 0;
        }
        
        self.updates_in_window += 1;
        self.last_update_us = current_time_us;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Scale;
    
    fn test_bounds() -> HashMap<String, Domain> {
        HashMap::from([
            ("x".to_string(), Domain { min: 0.0, max: 1.0, scale: Scale::Linear }),
            ("y".to_string(), Domain { min: -1.0, max: 1.0, scale: Scale::Linear }),
        ])
    }
    
    #[test]
    fn test_validate_delta_ok() {
        let safety = SafetyExecutor::new(Guardrails::default(), test_bounds());
        let current = HashMap::from([("x".to_string(), 0.5), ("y".to_string(), 0.0)]);
        let delta = HashMap::from([("x".to_string(), 0.05)]);
        
        assert!(safety.validate_delta(&current, &delta).is_ok());
    }
    
    #[test]
    fn test_validate_delta_too_large() {
        let safety = SafetyExecutor::new(Guardrails::default(), test_bounds());
        let current = HashMap::from([("x".to_string(), 0.5)]);
        let delta = HashMap::from([("x".to_string(), 0.5)]); // 50% > 10% max
        
        let result = safety.validate_delta(&current, &delta);
        assert!(matches!(result, Err(Violation::DeltaTooLarge { .. })));
    }
    
    #[test]
    fn test_validate_delta_out_of_bounds() {
        let safety = SafetyExecutor::new(Guardrails::default(), test_bounds());
        let current = HashMap::from([("x".to_string(), 0.95)]);
        let delta = HashMap::from([("x".to_string(), 0.1)]); // Would be 1.05 > 1.0
        
        let result = safety.validate_delta(&current, &delta);
        assert!(matches!(result, Err(Violation::OutOfBounds { .. })));
    }
    
    #[test]
    fn test_validate_unknown_param() {
        let safety = SafetyExecutor::new(Guardrails::default(), test_bounds());
        let current = HashMap::new();
        let delta = HashMap::from([("z".to_string(), 0.01)]);
        
        let result = safety.validate_delta(&current, &delta);
        assert!(matches!(result, Err(Violation::UnknownParameter { .. })));
    }
    
    #[test]
    fn test_baseline_rollback() {
        let mut safety = SafetyExecutor::new(Guardrails::default(), test_bounds());
        let baseline = HashMap::from([("x".to_string(), 0.5)]);
        
        safety.set_baseline(baseline.clone());
        assert_eq!(safety.baseline(), Some(baseline));
    }
}
