//! Control Safety: Anti-thrashing, stop-on-instability, and constraint-first policies.
//!
//! Constitution: II.23 - Unbounded exploration and oscillation (thrashing) are forbidden.

use crate::{
    config_atomic::{ParamId, ParamVec},
    executor::{Guardrails, Violation},
};

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
    // Dense tracking vectors (index = ParamId)
    direction_tracker: Vec<DirectionHistory>,
    delta_budget: Vec<DeltaBudget>,
    consecutive_regressions: u32,
    last_objective: Option<f64>,
    safe_mode: Option<SafeMode>,
}

impl ControlSafety {
    /// Create new control safety tracker.
    pub fn new(guardrails: Guardrails, num_params: usize) -> Self {
        Self {
            guardrails,
            direction_tracker: vec![DirectionHistory::default(); num_params],
            delta_budget: vec![DeltaBudget::default(); num_params],
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
            exit_condition: SafeModeExit::Timer {
                remaining_us: cooldown_us,
            },
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
            let direction: i8 = if d > 0.0 {
                1
            } else if d < 0.0 {
                -1
            } else {
                0
            };

            // Direction tracking
            let history = &mut self.direction_tracker[param_id as usize];

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
            let budget = &mut self.delta_budget[param_id as usize];

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
        let mut cs = ControlSafety::new(Guardrails::default(), 10);
        assert!(!cs.is_safe_mode());

        cs.enter_safe_mode(SafeModeReason::Thrashing, 1000, 30_000_000);
        assert!(cs.is_safe_mode());
        assert_eq!(cs.safe_mode().unwrap().reason, SafeModeReason::Thrashing);
    }

    #[test]
    fn test_safe_mode_timer_exit() {
        let mut cs = ControlSafety::new(Guardrails::default(), 10);
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
        let mut cs = ControlSafety::new(
            Guardrails {
                direction_flip_limit: 2,
                cooldown_after_flip_us: 1000,
                ..Default::default()
            },
            1,
        );

        cs.record_delta(&ParamVec::from_slice(&[0.05]), 1000);
        cs.record_delta(&ParamVec::from_slice(&[-0.05]), 2000); // flip 1
        cs.record_delta(&ParamVec::from_slice(&[0.05]), 3000); // flip 2

        assert!(!cs.is_safe_mode());

        cs.record_delta(&ParamVec::from_slice(&[-0.05]), 4000); // flip 3 → SafeMode
        assert!(cs.is_safe_mode());
    }

    #[test]
    fn test_reset_safe_mode() {
        let mut cs = ControlSafety::new(Guardrails::default(), 10);
        cs.enter_safe_mode(SafeModeReason::Thrashing, 1000, 30_000_000);
        assert!(cs.is_safe_mode());

        cs.reset_safe_mode();
        assert!(!cs.is_safe_mode());
    }

    #[test]
    fn test_check_proposal_in_safe_mode() {
        let mut cs = ControlSafety::new(Guardrails::default(), 2);
        cs.enter_safe_mode(SafeModeReason::Thrashing, 1000, 30_000_000);

        let delta = ParamVec::from_slice(&[0.1, 0.1]);
        let result = cs.check_proposal(&delta, 2000);

        // Should be Ok even in SafeMode (no-change indication)
        assert!(result.is_ok());
        assert!(cs.is_safe_mode());
    }

    #[test]
    fn test_check_proposal_exits_safe_mode_on_timer() {
        let mut cs = ControlSafety::new(Guardrails::default(), 2);
        cs.enter_safe_mode(SafeModeReason::Thrashing, 1000, 100);

        let delta = ParamVec::from_slice(&[0.1, 0.1]);
        // After timer expires
        let result = cs.check_proposal(&delta, 2000);

        assert!(result.is_ok());
        assert!(!cs.is_safe_mode()); // Should have exited
    }

    #[test]
    fn test_record_objective_regression() {
        let mut cs = ControlSafety::new(
            Guardrails {
                regression_count_limit: 2,
                cooldown_after_flip_us: 1000,
                ..Default::default()
            },
            1,
        );

        cs.record_objective(0.5, 1000); // First value
        cs.record_objective(0.6, 2000); // Worse (regression 1)
        assert!(!cs.is_safe_mode());

        cs.record_objective(0.7, 3000); // Worse (regression 2) → SafeMode
        assert!(cs.is_safe_mode());
        assert_eq!(
            cs.safe_mode().unwrap().reason,
            SafeModeReason::ObjectiveRegression
        );
    }

    #[test]
    fn test_record_objective_improvement_resets() {
        let mut cs = ControlSafety::new(
            Guardrails {
                regression_count_limit: 3,
                cooldown_after_flip_us: 1000,
                ..Default::default()
            },
            1,
        );

        cs.record_objective(0.5, 1000);
        cs.record_objective(0.6, 2000); // Worse (regression 1)
        cs.record_objective(0.7, 3000); // Worse (regression 2)
        cs.record_objective(0.3, 4000); // Better - resets counter
        cs.record_objective(0.4, 5000); // Worse (regression 1 again)

        assert!(!cs.is_safe_mode()); // Not in safe mode since counter was reset
    }

    #[test]
    fn test_budget_exhaustion() {
        let mut cs = ControlSafety::new(
            Guardrails {
                max_cumulative_delta_per_minute: 0.1,
                cooldown_after_flip_us: 1000,
                ..Default::default()
            },
            1,
        );

        // Keep adding deltas until budget exhausted
        cs.record_delta(&ParamVec::from_slice(&[0.05]), 1000);
        assert!(!cs.is_safe_mode());

        cs.record_delta(&ParamVec::from_slice(&[0.06]), 2000); // 0.11 > 0.1
        assert!(cs.is_safe_mode());
        assert_eq!(
            cs.safe_mode().unwrap().reason,
            SafeModeReason::BudgetExhausted
        );
    }

    #[test]
    fn test_delta_window_reset() {
        let mut cs = ControlSafety::new(
            Guardrails {
                max_cumulative_delta_per_minute: 0.1,
                cooldown_after_flip_us: 1000,
                ..Default::default()
            },
            1,
        );

        cs.record_delta(&ParamVec::from_slice(&[0.05]), 1000);
        assert!(!cs.is_safe_mode());

        // After 1 minute, budget resets
        let minute_plus = 1000 + 60_000_001;
        cs.record_delta(&ParamVec::from_slice(&[0.05]), minute_plus);
        assert!(!cs.is_safe_mode()); // Budget reset
    }

    #[test]
    fn test_direction_window_reset() {
        let mut cs = ControlSafety::new(
            Guardrails {
                direction_flip_limit: 2,
                cooldown_after_flip_us: 1000,
                ..Default::default()
            },
            1,
        );

        cs.record_delta(&ParamVec::from_slice(&[0.05]), 1000);
        cs.record_delta(&ParamVec::from_slice(&[-0.05]), 2000); // flip 1
        cs.record_delta(&ParamVec::from_slice(&[0.05]), 3000); // flip 2

        // After 1 minute, flip count resets
        let minute_plus = 3000 + 60_000_001;
        cs.record_delta(&ParamVec::from_slice(&[-0.05]), minute_plus); // flip 1 (reset)
        assert!(!cs.is_safe_mode());
    }

    #[test]
    fn test_zero_delta_no_direction_change() {
        let mut cs = ControlSafety::new(
            Guardrails {
                direction_flip_limit: 1,
                cooldown_after_flip_us: 1000,
                ..Default::default()
            },
            1,
        );

        cs.record_delta(&ParamVec::from_slice(&[0.05]), 1000);
        cs.record_delta(&ParamVec::from_slice(&[0.0]), 2000); // zero - no direction
        cs.record_delta(&ParamVec::from_slice(&[-0.05]), 3000); // flip 1

        // With limit 1, this should trigger SafeMode
        cs.record_delta(&ParamVec::from_slice(&[0.05]), 4000); // flip 2
        assert!(cs.is_safe_mode());
    }

    #[test]
    fn test_safe_mode_get() {
        let mut cs = ControlSafety::new(Guardrails::default(), 10);
        assert!(cs.safe_mode().is_none());

        cs.enter_safe_mode(SafeModeReason::ManualTrigger, 1000, 100);
        let mode = cs.safe_mode().unwrap();
        assert_eq!(mode.entered_at_us, 1000);
        assert_eq!(mode.reason, SafeModeReason::ManualTrigger);
    }
}
