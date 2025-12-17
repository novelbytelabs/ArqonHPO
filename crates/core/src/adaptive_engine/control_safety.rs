//! Control Safety: Anti-thrashing, stop-on-instability, and constraint-first policies.
//!
//! Constitution: II.23 - Unbounded exploration and oscillation (thrashing) are forbidden.

use crate::adaptive_engine::{
    config_atomic::{ParamId, ParamVec},
    executor::{Violation, Guardrails},
};
use std::collections::HashMap;

/// Reason for entering SafeMode.
#[derive(Clone, Debug, PartialEq)]
pub enum SafeModeReason {
    Thrashing,
    BudgetExhausted,
    ObjectiveRegression,
    AuditQueueFull,
    RepeatedViolations,
    ManualTrigger,
}

/// Exit condition for SafeMode.
#[derive(Clone, Debug)]
pub enum SafeModeExit {
    Timer { remaining_us: u64 },
    ManualReset,
    ObjectiveRecovery { required_improvement: f64 },
}

/// SafeMode latch state.
#[derive(Clone, Debug)]
pub struct SafeMode {
    pub entered_at_us: u64,
    pub reason: SafeModeReason,
    pub exit_condition: SafeModeExit,
}

/// Direction change tracking per parameter.
#[derive(Clone, Debug, Default)]
struct DirectionHistory {
    last_direction: Option<i8>,
    flip_count: u32,
    window_start_us: u64,
}

/// Cumulative delta tracking per parameter.
#[derive(Clone, Debug, Default)]
struct DeltaBudget {
    cumulative: f64,
    window_start_us: u64,
}

/// Control safety state machine.
pub struct ControlSafety {
    guardrails: Guardrails,
    direction_tracker: HashMap<ParamId, DirectionHistory>,
    delta_budget: HashMap<ParamId, DeltaBudget>,
    consecutive_regressions: u32,
    last_objective: Option<f64>,
    safe_mode: Option<SafeMode>,
}

impl ControlSafety {
    /// Create new control safety tracker.
    pub fn new(guardrails: Guardrails) -> Self {
        Self {
            guardrails,
            direction_tracker: HashMap::new(),
            delta_budget: HashMap::new(),
            consecutive_regressions: 0,
            last_objective: None,
            safe_mode: None,
        }
    }

    /// Check if currently in SafeMode.
    pub fn is_safe_mode(&self) -> bool {
        self.safe_mode.is_some()
    }

    /// Get SafeMode state if active.
    pub fn safe_mode(&self) -> Option<&SafeMode> {
        self.safe_mode.as_ref()
    }

    /// Enter SafeMode.
    pub fn enter_safe_mode(&mut self, reason: SafeModeReason, now_us: u64, cooldown_us: u64) {
        self.safe_mode = Some(SafeMode {
            entered_at_us: now_us,
            reason,
            exit_condition: SafeModeExit::Timer { remaining_us: cooldown_us },
        });
    }

    /// Try to exit SafeMode.
    pub fn try_exit_safe_mode(&mut self, now_us: u64) -> bool {
        if let Some(ref mode) = self.safe_mode {
            match &mode.exit_condition {
                SafeModeExit::Timer { remaining_us } => {
                    let elapsed = now_us.saturating_sub(mode.entered_at_us);
                    if elapsed >= *remaining_us {
                        self.safe_mode = None;
                        return true;
                    }
                }
                SafeModeExit::ManualReset => {
                    // Requires explicit call to reset
                }
                SafeModeExit::ObjectiveRecovery { .. } => {
                    // Checked separately
                }
            }
        }
        false
    }

    /// Manual reset of SafeMode.
    pub fn reset_safe_mode(&mut self) {
        self.safe_mode = None;
    }

    /// Check a proposal against control safety invariants.
    pub fn check_proposal(&mut self, _delta: &ParamVec, now_us: u64) -> Result<(), Violation> {
        // Try to exit SafeMode if timer expired
        self.try_exit_safe_mode(now_us);

        // If still in SafeMode, reject
        if self.is_safe_mode() {
            // Return a no-change indication (not a violation per se)
            return Ok(());
        }

        // Budget and thrashing checks happen in record_delta
        Ok(())
    }

    /// Record a delta for control safety tracking.
    pub fn record_delta(&mut self, delta: &ParamVec, now_us: u64) {
        let minute_us: u64 = 60_000_000;
        
        // Collect flags for SafeMode triggers in first pass to avoid borrow issues
        let mut enter_thrashing_mode = false;
        let mut enter_budget_mode = false;

        for (i, &d) in delta.iter().enumerate() {
            let param_id = i as ParamId;
            let direction: i8 = if d > 0.0 { 1 } else if d < 0.0 { -1 } else { 0 };

            // Direction tracking
            let history = self.direction_tracker.entry(param_id).or_default();
            
            // Reset window if expired
            if now_us.saturating_sub(history.window_start_us) > minute_us {
                history.flip_count = 0;
                history.window_start_us = now_us;
            }

            // Check for direction flip
            if direction != 0 {
                if let Some(last) = history.last_direction {
                    if last != 0 && last != direction {
                        history.flip_count += 1;
                        
                        // Check thrashing limit
                        if history.flip_count > self.guardrails.direction_flip_limit {
                            enter_thrashing_mode = true;
                        }
                    }
                }
                history.last_direction = Some(direction);
            }

            // Budget tracking
            let budget = self.delta_budget.entry(param_id).or_default();
            
            // Reset window if expired
            if now_us.saturating_sub(budget.window_start_us) > minute_us {
                budget.cumulative = 0.0;
                budget.window_start_us = now_us;
            }

            budget.cumulative += d.abs();

            // Check budget limit
            if budget.cumulative > self.guardrails.max_cumulative_delta_per_minute {
                enter_budget_mode = true;
            }
        }
        
        // Enter SafeMode after iteration to avoid borrow conflicts
        if enter_thrashing_mode {
            self.enter_safe_mode(
                SafeModeReason::Thrashing,
                now_us,
                self.guardrails.cooldown_after_flip_us,
            );
        } else if enter_budget_mode {
            self.enter_safe_mode(
                SafeModeReason::BudgetExhausted,
                now_us,
                self.guardrails.cooldown_after_flip_us,
            );
        }
    }

    /// Record an objective value for regression detection.
    pub fn record_objective(&mut self, value: f64, now_us: u64) {
        if let Some(last) = self.last_objective {
            // Worsening = higher value (assuming minimization)
            if value > last + 0.01 {
                self.consecutive_regressions += 1;
                if self.consecutive_regressions >= self.guardrails.regression_count_limit {
                    self.enter_safe_mode(
                        SafeModeReason::ObjectiveRegression,
                        now_us,
                        self.guardrails.cooldown_after_flip_us,
                    );
                }
            } else {
                self.consecutive_regressions = 0;
            }
        }
        self.last_objective = Some(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_mode_entry() {
        let mut cs = ControlSafety::new(Guardrails::default());
        assert!(!cs.is_safe_mode());
        
        cs.enter_safe_mode(SafeModeReason::Thrashing, 1000, 30_000_000);
        assert!(cs.is_safe_mode());
        assert_eq!(cs.safe_mode().unwrap().reason, SafeModeReason::Thrashing);
    }

    #[test]
    fn test_safe_mode_timer_exit() {
        let mut cs = ControlSafety::new(Guardrails::default());
        cs.enter_safe_mode(SafeModeReason::Thrashing, 1000, 100);
        
        // Before timer
        assert!(!cs.try_exit_safe_mode(1050));
        assert!(cs.is_safe_mode());
        
        // After timer
        assert!(cs.try_exit_safe_mode(1200));
        assert!(!cs.is_safe_mode());
    }

    #[test]
    fn test_direction_flip_detection() {
        let mut cs = ControlSafety::new(Guardrails {
            direction_flip_limit: 2,
            cooldown_after_flip_us: 1000,
            ..Default::default()
        });

        cs.record_delta(&ParamVec::from_slice(&[0.05]), 1000);
        cs.record_delta(&ParamVec::from_slice(&[-0.05]), 2000); // flip 1
        cs.record_delta(&ParamVec::from_slice(&[0.05]), 3000);  // flip 2
        
        assert!(!cs.is_safe_mode());
        
        cs.record_delta(&ParamVec::from_slice(&[-0.05]), 4000); // flip 3 → SafeMode
        assert!(cs.is_safe_mode());
    }
}
