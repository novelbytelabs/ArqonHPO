//! Adaptive Proposer trait and implementations.
//!
//! Constitution: II.20 - Tier 2 MUST NOT directly mutate production state.

use crate::adaptive_engine::{
    config_atomic::ParamVec,
    telemetry::TelemetryDigest,
};

/// Error from proposal generation.
#[derive(Clone, Debug)]
pub enum ProposalError {
    InvalidDigest(String),
    InternalError(String),
}

/// Reason for NoChange proposal.
#[derive(Clone, Debug, PartialEq)]
pub enum NoChangeReason {
    EvalTimeout,
    SafeMode,
    ConstraintViolation,
    CooldownActive,
    BudgetExhausted,
}

/// Proposal from Tier 2 to Tier 1.
#[derive(Clone, Debug)]
pub enum Proposal {
    /// Apply +Δ perturbation for y+ evaluation.
    ApplyPlus {
        perturbation_id: u64,
        delta: ParamVec,
    },
    /// Apply −Δ perturbation for y− evaluation.
    ApplyMinus {
        perturbation_id: u64,
        delta: ParamVec,
    },
    /// Apply real gradient-based update.
    Update {
        iteration: u64,
        delta: ParamVec,
        gradient_estimate: ParamVec,
    },
    /// No change (timeout, safe mode, etc.).
    NoChange {
        reason: NoChangeReason,
    },
}

/// Result of observing telemetry.
pub type ProposalResult = Result<Proposal, ProposalError>;

/// Tier 2 proposal generator.
///
/// Constitution: II.20-21
/// - MUST NOT hold a reference to AtomicConfig
/// - MUST NOT call any method that mutates production state
/// - MUST be deterministic given same seed and digest stream
pub trait AdaptiveProposer {
    /// Observe a telemetry digest and potentially generate a proposal.
    fn observe(&mut self, digest: TelemetryDigest) -> ProposalResult;
    
    /// Get the current perturbation being evaluated (if any).
    fn current_perturbation(&self) -> Option<(u64, ParamVec)>;
    
    /// Get the current SPSA iteration count.
    fn iteration(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_variants() {
        let delta = ParamVec::from([0.1, 0.2]);
        
        let p = Proposal::ApplyPlus {
            perturbation_id: 1,
            delta: delta.clone(),
        };
        
        match p {
            Proposal::ApplyPlus { perturbation_id, .. } => {
                assert_eq!(perturbation_id, 1);
            }
            _ => panic!("Expected ApplyPlus"),
        }
    }
}
