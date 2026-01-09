#![deny(clippy::disallowed_types)]
#![deny(warnings)]

pub mod audit;
pub mod config_atomic;
pub mod control_safety;
pub mod executor;
pub mod homeostasis;
pub mod orchestrator;
pub mod proposer;
pub mod spsa;
pub mod telemetry;

// Re-exports for API compatibility with arqonhpo_core::adaptive_engine
pub use audit::{AuditEvent, AuditPolicy, AuditQueue, EnqueueResult, EventType};
pub use config_atomic::{
    param_vec, AtomicConfig, ConfigSnapshot, ParamId, ParamRegistry, ParamVec,
};
pub use control_safety::{ControlSafety, SafeMode, SafeModeExit, SafeModeReason};
pub use executor::{
    ApplyReceipt, Guardrails, RollbackPolicy, RollbackReceipt, SafeExecutor, SafetyExecutor,
    Violation,
};
pub use orchestrator::{AdaptiveEngine, AdaptiveEngineConfig};
pub use proposer::{AdaptiveProposer, NoChangeReason, Proposal, ProposalResult};
pub use spsa::{Spsa, SpsaConfig, SpsaState};
pub use telemetry::{DigestValidity, TelemetryDigest, TelemetryRingBuffer};
