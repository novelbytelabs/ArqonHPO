//! Adaptive Engine Trait Contracts
//!
//! This file defines the tier boundary contracts for the Adaptive Engine.
//! These are the authoritative API definitions from the spec.

use std::collections::HashMap;

// ============================================================================
// TIER 2: ADAPTIVE PROPOSER (spec §3.1)
// ============================================================================

/// Tier 2 proposal generator.
///
/// # Contract
/// - MUST NOT hold a reference to `AtomicConfig`
/// - MUST NOT call any method that mutates production state
/// - MUST be deterministic given same seed and digest stream
pub trait AdaptiveProposer {
    /// Observe a telemetry digest and potentially generate a proposal.
    ///
    /// # Arguments
    /// * `digest` - Compact telemetry from data plane
    ///
    /// # Returns
    /// * `ProposalResult` - The proposal or reason for no change
    fn observe(&mut self, digest: TelemetryDigest) -> ProposalResult;
    
    /// Get the current perturbation being evaluated (if any).
    fn current_perturbation(&self) -> Option<Perturbation>;
    
    /// Get the current SPSA iteration count.
    fn iteration(&self) -> u64;
}

// ============================================================================
// TIER 1: SAFE EXECUTOR (spec §3.1, II.17)
// ============================================================================

/// Tier 1 safe executor.
///
/// # Contract
/// - SOLE actuator that may modify production config
/// - MUST validate all proposals through guardrails
/// - MUST reject proposals that violate safety invariants
/// - MUST preserve baseline for rollback
pub trait SafeExecutor {
    /// Apply a proposal through safety guardrails.
    ///
    /// # Arguments
    /// * `proposal` - The proposal from Tier 2
    ///
    /// # Returns
    /// * `Ok(ApplyReceipt)` - Successfully applied, with new generation
    /// * `Err(Violation)` - Rejected by guardrails
    fn apply(&mut self, proposal: Proposal) -> Result<ApplyReceipt, Violation>;
    
    /// Rollback to baseline configuration.
    ///
    /// # Returns
    /// * `Ok(RollbackReceipt)` - Successfully rolled back
    /// * `Err(Violation)` - Rollback failed (no baseline set)
    fn rollback(&mut self) -> Result<RollbackReceipt, Violation>;
    
    /// Set current config as baseline for future rollbacks.
    fn set_baseline(&mut self);
    
    /// Get current config snapshot (zero-copy).
    fn snapshot(&self) -> ConfigSnapshot;
}

// ============================================================================
// DATA TYPES (spec §6, §7)
// ============================================================================

/// Dense parameter vector (stack-allocated for ≤16 params).
pub type ParamVec = smallvec::SmallVec<[f64; 16]>;

/// Stable parameter identifier.
pub type ParamId = u16;

/// Immutable configuration snapshot.
#[derive(Clone, Debug)]
pub struct ConfigSnapshot {
    pub params: ParamVec,
    pub generation: u64,
}

/// Compact telemetry digest (≤128 bytes).
#[derive(Clone, Debug, Default)]
pub struct TelemetryDigest {
    pub timestamp_us: u64,
    pub objective_value: f64,
    pub config_generation: u64,
    pub latency_p99_us: Option<u64>,
    pub throughput_rps: Option<f64>,
    pub error_rate: Option<f64>,
    pub constraint_margin: Option<f64>,
}

/// SPSA perturbation being evaluated.
#[derive(Clone, Debug)]
pub struct Perturbation {
    pub id: u64,
    pub delta: ParamVec,
    pub direction: PerturbationDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PerturbationDirection {
    Plus,
    Minus,
}

// ============================================================================
// PROPOSAL TYPES (spec §4.5)
// ============================================================================

/// Result of observing telemetry.
pub type ProposalResult = Result<Proposal, ProposalError>;

/// Proposal from Tier 2 to Tier 1.
#[derive(Clone, Debug)]
pub enum Proposal {
    /// Apply +Δ perturbation for y+ evaluation
    ApplyPlus { perturbation_id: u64, delta: ParamVec },
    /// Apply −Δ perturbation for y− evaluation
    ApplyMinus { perturbation_id: u64, delta: ParamVec },
    /// Apply real gradient-based update
    Update { iteration: u64, delta: ParamVec, gradient_estimate: ParamVec },
    /// No change (timeout, safe mode, etc.)
    NoChange { reason: NoChangeReason },
}

#[derive(Clone, Debug)]
pub enum NoChangeReason {
    EvalTimeout,
    SafeMode,
    ConstraintViolation,
    CooldownActive,
    BudgetExhausted,
}

#[derive(Clone, Debug)]
pub enum ProposalError {
    InvalidDigest(String),
    InternalError(String),
}

// ============================================================================
// SAFETY TYPES (spec §5, §11.2)
// ============================================================================

/// Safety violation preventing apply.
#[derive(Clone, Debug)]
pub enum Violation {
    DeltaTooLarge { param_id: ParamId, delta: f64, max: f64 },
    RateLimitExceeded { rate: f64, max: f64 },
    OutOfBounds { param_id: ParamId, value: f64, min: f64, max: f64 },
    UnknownParameter { param_id: ParamId },
    Thrashing { param_id: ParamId, flips: u32, limit: u32 },
    BudgetExhausted { used: f64, limit: f64 },
    ObjectiveRegression { count: u32, limit: u32 },
    ConstraintViolation { margin: f64 },
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

// ============================================================================
// CONFIGURATION (spec §11.4)
// ============================================================================

/// Guardrails configuration.
#[derive(Clone, Debug)]
pub struct Guardrails {
    pub max_delta_per_step: f64,
    pub max_updates_per_second: f64,
    pub min_interval_us: u64,
    pub direction_flip_limit: u32,
    pub cooldown_after_flip_us: u64,
    pub max_cumulative_delta_per_minute: f64,
    pub regression_count_limit: u32,
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
        }
    }
}

/// Objective aggregation method.
#[derive(Clone, Debug)]
pub enum AggregationMethod {
    Mean,
    Median,
    TrimmedMean { trim_percent: f64 },
}

impl Default for AggregationMethod {
    fn default() -> Self {
        Self::TrimmedMean { trim_percent: 0.1 }
    }
}
