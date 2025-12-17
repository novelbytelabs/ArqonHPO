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
pub use config_atomic::{ParamId, ParamVec, ParamRegistry, ConfigSnapshot, AtomicConfig, param_vec};
pub use telemetry::{TelemetryDigest, TelemetryRingBuffer, DigestValidity};
pub use spsa::{Spsa, SpsaConfig, SpsaState};
pub use proposer::{AdaptiveProposer, Proposal, ProposalResult, NoChangeReason};
pub use executor::{
    SafeExecutor, SafetyExecutor, Guardrails, Violation, 
    ApplyReceipt, RollbackReceipt
};
pub use control_safety::{ControlSafety, SafeMode, SafeModeReason, SafeModeExit};
pub use audit::{AuditQueue, AuditEvent, EventType, AuditPolicy};
pub use orchestrator::{AdaptiveEngine, AdaptiveEngineConfig};
