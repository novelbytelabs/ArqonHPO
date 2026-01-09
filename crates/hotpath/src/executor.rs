//! Safety Executor for Tier 1 guardrails.
//!
//! Constitution: II.17 - All updates MUST pass through SafetyExecutor.

use crate::{
    config_atomic::{AtomicConfig, ConfigSnapshot, ParamId, ParamVec},
    control_safety::ControlSafety,
    proposer::Proposal,
};
use std::sync::Arc;
use std::time::Instant;

/// Safety violation preventing apply.
#[derive(Clone, Debug)]
pub enum Violation {
    DeltaTooLarge {
        param_id: ParamId,
        delta: f64,
        max: f64,
    },
    RateLimitExceeded {
        rate: f64,
        max: f64,
    },
    OutOfBounds {
        param_id: ParamId,
        value: f64,
        min: f64,
        max: f64,
    },
    UnknownParameter {
        param_id: ParamId,
    },
    Thrashing {
        param_id: ParamId,
        flips: u32,
        limit: u32,
    },
    BudgetExhausted {
        used: f64,
        limit: f64,
    },
    ObjectiveRegression {
        count: u32,
        limit: u32,
    },
    ConstraintViolation {
        margin: f64,
    },
    AuditQueueFull,
    NoBaseline,
}

/// Receipt from successful apply.
#[derive(Clone, Debug)]
pub struct ApplyReceipt {
    pub new_generation: u64,
    pub apply_latency_us: u64,
}

/// Receipt from successful rollback.
#[derive(Clone, Debug)]
pub struct RollbackReceipt {
    pub reverted_to_generation: u64,
    pub reason: String,
}

/// Guardrails configuration.
#[derive(Clone, Debug)]
pub struct Guardrails {
    /// Maximum delta per parameter per step (fraction).
    pub max_delta_per_step: f64,
    /// Maximum updates per second.
    pub max_updates_per_second: f64,
    /// Minimum interval between updates (microseconds).
    pub min_interval_us: u64,
    /// Maximum direction flips per dimension per minute.
    pub direction_flip_limit: u32,
    /// Cooldown after hitting flip limit (microseconds).
    pub cooldown_after_flip_us: u64,
    /// Maximum cumulative delta per dimension per minute.
    pub max_cumulative_delta_per_minute: f64,
    /// Consecutive regressions before SafeMode.
    pub regression_count_limit: u32,
    /// Per-parameter bounds: (min, max).
    pub bounds: Option<Vec<(f64, f64)>>,
}

impl Default for Guardrails {
    fn default() -> Self {
        Self {
            max_delta_per_step: 0.1,
            max_updates_per_second: 10.0,
            min_interval_us: 100_000,
            direction_flip_limit: 3,
            cooldown_after_flip_us: 30_000_000,
            max_cumulative_delta_per_minute: 0.5,
            regression_count_limit: 5,
            bounds: None,
        }
    }
}

impl Guardrails {
    pub fn preset_conservative() -> Self {
        Self {
            max_delta_per_step: 0.05,
            max_updates_per_second: 2.0,
            min_interval_us: 500_000,
            direction_flip_limit: 2,
            cooldown_after_flip_us: 60_000_000,
            max_cumulative_delta_per_minute: 0.25,
            regression_count_limit: 3,
            bounds: None,
        }
    }

    pub fn preset_balanced() -> Self {
        Self::default()
    }

    pub fn preset_aggressive() -> Self {
        Self {
            max_delta_per_step: 0.2,
            max_updates_per_second: 20.0,
            min_interval_us: 50_000,
            direction_flip_limit: 5,
            cooldown_after_flip_us: 10_000_000,
            max_cumulative_delta_per_minute: 1.0,
            regression_count_limit: 8,
            bounds: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RollbackPolicy {
    pub max_consecutive_regressions: u32,
    pub max_rollbacks_per_hour: u32,
    pub min_stable_time_us: u64,
}

impl Default for RollbackPolicy {
    fn default() -> Self {
        Self {
            max_consecutive_regressions: 3,
            max_rollbacks_per_hour: 4,
            min_stable_time_us: 5_000_000,
        }
    }
}

impl RollbackPolicy {
    pub fn preset_conservative() -> Self {
        Self {
            max_consecutive_regressions: 2,
            max_rollbacks_per_hour: 2,
            min_stable_time_us: 30_000_000,
        }
    }

    pub fn preset_balanced() -> Self {
        Self::default()
    }

    pub fn preset_aggressive() -> Self {
        Self {
            max_consecutive_regressions: 5,
            max_rollbacks_per_hour: 10,
            min_stable_time_us: 1_000_000,
        }
    }
}

/// Tier 1 safe executor.
///
/// Constitution: II.17 - Safety Executor Contract
/// - SOLE actuator that may modify production config
/// - MUST validate all proposals through guardrails  
/// - MUST reject proposals that violate safety invariants
/// - MUST preserve baseline for rollback
pub trait SafeExecutor {
    /// Apply a proposal through safety guardrails.
    fn apply(&mut self, proposal: Proposal) -> Result<ApplyReceipt, Violation>;

    /// Rollback to baseline configuration.
    fn rollback(&mut self) -> Result<RollbackReceipt, Violation>;

    /// Set current config as baseline for future rollbacks.
    fn set_baseline(&mut self);

    /// Get current config snapshot (zero-copy).
    fn snapshot(&self) -> ConfigSnapshot;
}

/// Safety executor implementation.
pub struct SafetyExecutor {
    config: Arc<AtomicConfig>,
    guardrails: Guardrails,
    control_safety: ControlSafety,
    last_apply_us: u64,
    update_count_window: Vec<u64>,
}

impl SafetyExecutor {
    /// Create a new safety executor.
    pub fn new(config: Arc<AtomicConfig>, guardrails: Guardrails) -> Self {
        let num_params = config.snapshot().params.len();
        Self {
            config,
            guardrails: guardrails.clone(),
            control_safety: ControlSafety::new(guardrails, num_params),
            last_apply_us: 0,
            update_count_window: Vec::new(),
        }
    }

    /// Validate a delta against guardrails.
    pub fn validate_delta(&self, delta: &ParamVec, current: &ParamVec) -> Result<(), Violation> {
        for (i, (&d, &c)) in delta.iter().zip(current.iter()).enumerate() {
            let param_id = i as ParamId;

            // Check delta magnitude
            if d.abs() > self.guardrails.max_delta_per_step {
                return Err(Violation::DeltaTooLarge {
                    param_id,
                    delta: d.abs(),
                    max: self.guardrails.max_delta_per_step,
                });
            }

            // Check bounds
            if let Some(ref bounds) = self.guardrails.bounds {
                if let Some(&(min, max)) = bounds.get(i) {
                    let new_value = c + d;
                    if new_value < min || new_value > max {
                        return Err(Violation::OutOfBounds {
                            param_id,
                            value: new_value,
                            min,
                            max,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Clamp values to bounds.
    pub fn clamp_to_bounds(&self, params: &mut ParamVec) {
        if let Some(ref bounds) = self.guardrails.bounds {
            for (i, value) in params.iter_mut().enumerate() {
                if let Some(&(min, max)) = bounds.get(i) {
                    *value = value.clamp(min, max);
                }
            }
        }
    }

    fn get_timestamp_us() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
}

impl SafeExecutor for SafetyExecutor {
    fn apply(&mut self, proposal: Proposal) -> Result<ApplyReceipt, Violation> {
        let start = Instant::now();
        let now_us = Self::get_timestamp_us();

        // Rate limiting
        let window_start = now_us.saturating_sub(1_000_000);
        self.update_count_window.retain(|&t| t > window_start);
        let rate = self.update_count_window.len() as f64;
        if rate >= self.guardrails.max_updates_per_second {
            return Err(Violation::RateLimitExceeded {
                rate,
                max: self.guardrails.max_updates_per_second,
            });
        }

        // Extract delta from proposal
        let delta = match &proposal {
            Proposal::ApplyPlus { delta, .. }
            | Proposal::ApplyMinus { delta, .. }
            | Proposal::Update { delta, .. } => delta.clone(),
            Proposal::NoChange { .. } => {
                // No-op
                return Ok(ApplyReceipt {
                    new_generation: self.config.generation(),
                    apply_latency_us: start.elapsed().as_micros() as u64,
                });
            }
        };

        // Get current config
        let current = self.config.snapshot();

        // Validate delta
        self.validate_delta(&delta, &current.params)?;

        // Check control safety
        self.control_safety.check_proposal(&delta, now_us)?;

        // Apply delta
        let mut new_params = current.params.clone();
        for (i, &d) in delta.iter().enumerate() {
            if i < new_params.len() {
                new_params[i] += d;
            }
        }
        self.clamp_to_bounds(&mut new_params);

        // Atomic swap
        let new_gen = self.config.swap(new_params);

        // Track for rate limiting
        self.update_count_window.push(now_us);
        self.last_apply_us = now_us;

        // Record delta for control safety
        self.control_safety.record_delta(&delta, now_us);

        Ok(ApplyReceipt {
            new_generation: new_gen,
            apply_latency_us: start.elapsed().as_micros() as u64,
        })
    }

    fn rollback(&mut self) -> Result<RollbackReceipt, Violation> {
        match self.config.rollback() {
            Some(new_gen) => Ok(RollbackReceipt {
                reverted_to_generation: new_gen,
                reason: "Manual rollback".to_string(),
            }),
            None => Err(Violation::NoBaseline),
        }
    }

    fn set_baseline(&mut self) {
        self.config.set_baseline();
    }

    fn snapshot(&self) -> ConfigSnapshot {
        (*self.config.snapshot()).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardrails_default() {
        let g = Guardrails::default();
        assert_eq!(g.max_delta_per_step, 0.1);
        assert_eq!(g.max_updates_per_second, 10.0);
    }

    #[test]
    fn test_validate_delta_too_large() {
        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let executor = SafetyExecutor::new(config, Guardrails::default());

        let delta = ParamVec::from_slice(&[0.5, 0.0]); // 0.5 > 0.1 max
        let current = ParamVec::from_slice(&[0.5, 0.5]);

        let result = executor.validate_delta(&delta, &current);
        assert!(matches!(result, Err(Violation::DeltaTooLarge { .. })));
    }

    #[test]
    fn test_validate_delta_ok() {
        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let executor = SafetyExecutor::new(config, Guardrails::default());

        let delta = ParamVec::from_slice(&[0.05, 0.05]); // Within 0.1 limit
        let current = ParamVec::from_slice(&[0.5, 0.5]);

        let result = executor.validate_delta(&delta, &current);
        assert!(result.is_ok());
    }

    #[test]
    fn test_guardrails_preset_conservative() {
        let g = Guardrails::preset_conservative();
        assert_eq!(g.max_delta_per_step, 0.05);
        assert_eq!(g.max_updates_per_second, 2.0);
        assert_eq!(g.direction_flip_limit, 2);
    }

    #[test]
    fn test_guardrails_preset_balanced() {
        let g = Guardrails::preset_balanced();
        assert_eq!(g.max_delta_per_step, 0.1);
        assert_eq!(g.max_updates_per_second, 10.0);
    }

    #[test]
    fn test_guardrails_preset_aggressive() {
        let g = Guardrails::preset_aggressive();
        assert_eq!(g.max_delta_per_step, 0.2);
        assert_eq!(g.max_updates_per_second, 20.0);
        assert_eq!(g.direction_flip_limit, 5);
    }

    #[test]
    fn test_rollback_policy_default() {
        let p = RollbackPolicy::default();
        assert_eq!(p.max_consecutive_regressions, 3);
        assert_eq!(p.max_rollbacks_per_hour, 4);
    }

    #[test]
    fn test_rollback_policy_presets() {
        let conservative = RollbackPolicy::preset_conservative();
        assert_eq!(conservative.max_consecutive_regressions, 2);

        let balanced = RollbackPolicy::preset_balanced();
        assert_eq!(balanced.max_consecutive_regressions, 3);

        let aggressive = RollbackPolicy::preset_aggressive();
        assert_eq!(aggressive.max_consecutive_regressions, 5);
    }

    #[test]
    fn test_clamp_to_bounds() {
        let mut guardrails = Guardrails::default();
        guardrails.bounds = Some(vec![(0.0, 1.0), (0.2, 0.8)]);

        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let executor = SafetyExecutor::new(config, guardrails);

        let mut params = ParamVec::from_slice(&[-0.5, 1.5]); // Out of bounds
        executor.clamp_to_bounds(&mut params);

        assert_eq!(params[0], 0.0); // Clamped to min
        assert_eq!(params[1], 0.8); // Clamped to max
    }

    #[test]
    fn test_validate_delta_out_of_bounds() {
        let mut guardrails = Guardrails::default();
        guardrails.bounds = Some(vec![(0.0, 1.0), (0.0, 1.0)]);

        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.9, 0.5])));
        let executor = SafetyExecutor::new(config, guardrails);

        let delta = ParamVec::from_slice(&[0.05, 0.0]); // Would push to 0.95
        let current = ParamVec::from_slice(&[0.9, 0.5]);
        assert!(executor.validate_delta(&delta, &current).is_ok());

        let delta_bad = ParamVec::from_slice(&[0.05, 0.0]); // Test at 0.96, should be OK
        let current_edge = ParamVec::from_slice(&[0.96, 0.5]); // 0.96 + 0.05 = 1.01 > 1.0
        let result = executor.validate_delta(&delta_bad, &current_edge);
        assert!(matches!(result, Err(Violation::OutOfBounds { .. })));
    }

    #[test]
    fn test_snapshot_returns_current_config() {
        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.3, 0.7])));
        let executor = SafetyExecutor::new(config, Guardrails::default());

        let snapshot = executor.snapshot();
        assert_eq!(snapshot.params.len(), 2);
        assert_eq!(snapshot.params[0], 0.3);
        assert_eq!(snapshot.params[1], 0.7);
    }

    #[test]
    fn test_rollback_no_baseline() {
        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());

        let result = executor.rollback();
        assert!(matches!(result, Err(Violation::NoBaseline)));
    }

    #[test]
    fn test_set_baseline_and_rollback() {
        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());

        executor.set_baseline();

        // Rollback should now succeed
        let result = executor.rollback();
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_update_proposal() {
        use crate::proposer::Proposal;

        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());

        let proposal = Proposal::Update {
            iteration: 1,
            delta: ParamVec::from_slice(&[0.05, -0.05]),
            gradient_estimate: ParamVec::from_slice(&[0.1, -0.1]),
        };

        let result = executor.apply(proposal);
        assert!(result.is_ok());

        let receipt = result.unwrap();
        assert_eq!(receipt.new_generation, 1);
    }

    #[test]
    fn test_apply_no_change_proposal() {
        use crate::proposer::{Proposal, NoChangeReason};

        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());

        let proposal = Proposal::NoChange {
            reason: NoChangeReason::CooldownActive,
        };

        let result = executor.apply(proposal);
        assert!(result.is_ok());

        let receipt = result.unwrap();
        // Generation should stay at 0 since no change
        assert_eq!(receipt.new_generation, 0);
    }

    #[test]
    fn test_apply_plus_proposal() {
        use crate::proposer::Proposal;

        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());

        let proposal = Proposal::ApplyPlus {
            perturbation_id: 1,
            delta: ParamVec::from_slice(&[0.02, 0.0]),
        };

        let result = executor.apply(proposal);
        assert!(result.is_ok());

        let snapshot = executor.snapshot();
        assert!((snapshot.params[0] - 0.52).abs() < 0.001);
    }

    #[test]
    fn test_apply_minus_proposal() {
        use crate::proposer::Proposal;

        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());

        let proposal = Proposal::ApplyMinus {
            perturbation_id: 1,
            delta: ParamVec::from_slice(&[-0.02, 0.0]),
        };

        let result = executor.apply(proposal);
        assert!(result.is_ok());

        let snapshot = executor.snapshot();
        assert!((snapshot.params[0] - 0.48).abs() < 0.001);
    }

    #[test]
    fn test_apply_delta_rejected_too_large() {
        use crate::proposer::Proposal;

        let config = Arc::new(AtomicConfig::new(ParamVec::from_slice(&[0.5, 0.5])));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());

        let proposal = Proposal::Update {
            iteration: 1,
            delta: ParamVec::from_slice(&[0.5, 0.0]), // 0.5 > 0.1 limit
            gradient_estimate: ParamVec::from_slice(&[0.5, 0.0]),
        };

        let result = executor.apply(proposal);
        assert!(matches!(result, Err(Violation::DeltaTooLarge { .. })));
    }
}
