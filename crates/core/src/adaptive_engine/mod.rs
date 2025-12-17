//! Adaptive Engine: Real-time parameter tuning at microsecond latency.
//!
//! This module implements the ArqonHPO Tier 2 Adaptive Engine, which provides:
//! - **SPSA optimization** for online gradient estimation with 2 evaluations per update
//! - **Safety guardrails** preventing unbounded changes and rollback support
//! - **Control safety** with anti-thrashing, stop-on-instability, and constraint-first policies
//! - **Audit completeness** with lock-free audit queue and no silent drops
//!
//! # Architecture
//!
//! ```text
//! Tier 2 (AdaptiveProposer)     Tier 1 (SafeExecutor)
//!        │                              │
//!        │  observe(digest)             │
//!        ├─────────────────────────────→│
//!        │                              │
//!        │  Proposal                    │
//!        │←─────────────────────────────┤
//!        │                              │
//!        │        apply(proposal)       │
//!        ├─────────────────────────────→│
//!        │                              │
//!        │  Result<Receipt, Violation>  │
//!        │←─────────────────────────────┤
//! ```
//!
//! # Constitution Reference
//!
//! - II.16-23: SPSA, Safety Executor, Atomic Config, Telemetry, Tier Architecture
//! - VIII.4-6: Timing contracts (T2_decision_us ≤ 1,000 µs)
//! - IX.2: Structured events with correlation IDs

mod config_atomic;
mod telemetry;
mod spsa;
mod proposer;
mod executor;
mod control_safety;
mod audit;
mod homeostasis;

// Re-exports
pub use config_atomic::{ParamId, ParamVec, ParamRegistry, ConfigSnapshot, AtomicConfig};
pub use telemetry::{TelemetryDigest, TelemetryRingBuffer, DigestValidity};
pub use spsa::{Spsa, SpsaConfig, SpsaState};
pub use proposer::{AdaptiveProposer, Proposal, ProposalResult, NoChangeReason};
pub use executor::{
    SafeExecutor, SafetyExecutor, Guardrails, Violation, 
    ApplyReceipt, RollbackReceipt
};
pub use control_safety::{SafeMode, SafeModeReason, SafeModeExit, ControlSafety};
pub use audit::{AuditEvent, AuditQueue, EventType, EnqueueResult};

use std::sync::Arc;

/// Configuration for the Adaptive Engine.
#[derive(Clone, Debug)]
pub struct AdaptiveEngineConfig {
    /// PRNG seed for reproducibility
    pub seed: u64,
    /// Initial learning rate (a₀)
    pub learning_rate: f64,
    /// Perturbation scale (c₀)
    pub perturbation_scale: f64,
    /// SPSA-specific configuration
    pub spsa: SpsaConfig,
    /// Safety guardrails
    pub guardrails: Guardrails,
    /// Telemetry ring buffer capacity
    pub telemetry_buffer_capacity: usize,
    /// Audit queue capacity
    pub audit_queue_capacity: usize,
}

impl Default for AdaptiveEngineConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            learning_rate: 0.1,
            perturbation_scale: 0.01,
            spsa: SpsaConfig::default(),
            guardrails: Guardrails::default(),
            telemetry_buffer_capacity: 1024,
            audit_queue_capacity: 4096,
        }
    }
}

/// The main Adaptive Engine orchestrator.
///
/// Wires together the proposer (Tier 2) and executor (Tier 1) with
/// telemetry buffering and audit logging.
pub struct AdaptiveEngine {
    proposer: Spsa,
    executor: SafetyExecutor,
    telemetry_buffer: TelemetryRingBuffer,
    audit_queue: AuditQueue,
    config: Arc<AtomicConfig>,
    run_id: u64,
}

impl AdaptiveEngine {
    /// Create a new Adaptive Engine with the given configuration and initial parameters.
    pub fn new(engine_config: AdaptiveEngineConfig, initial_params: ParamVec) -> Self {
        let num_params = initial_params.len();
        let config = Arc::new(AtomicConfig::new(initial_params));
        
        let proposer = Spsa::new(
            engine_config.seed,
            num_params,
            engine_config.learning_rate,
            engine_config.perturbation_scale,
            engine_config.spsa.clone(),
        );
        
        let executor = SafetyExecutor::new(config.clone(), engine_config.guardrails.clone());
        let telemetry_buffer = TelemetryRingBuffer::new(engine_config.telemetry_buffer_capacity);
        let audit_queue = AuditQueue::new(engine_config.audit_queue_capacity);
        
        Self {
            proposer,
            executor,
            telemetry_buffer,
            audit_queue,
            config,
            run_id: engine_config.seed,
        }
    }

    /// Observe a telemetry digest and potentially generate a proposal.
    /// 
    /// This is the main entry point for the adaptation loop:
    /// 1. Buffer the digest
    /// 2. Check if SPSA has enough samples
    /// 3. Generate proposal based on SPSA state
    pub fn observe(&mut self, digest: TelemetryDigest) -> ProposalResult {
        // Get current timestamp for audit
        let now_us = Self::get_timestamp_us();
        let config_gen = self.config.generation();
        
        // Emit audit event for digest
        let audit_result = self.audit_queue.enqueue(
            AuditEvent::new(EventType::Digest, now_us, self.run_id, config_gen)
        );
        
        // Check if audit queue is under pressure
        if audit_result == EnqueueResult::Full {
            return Ok(Proposal::NoChange { reason: NoChangeReason::SafeMode });
        }
        
        // Buffer the digest
        self.telemetry_buffer.push(digest.clone());
        
        // Check SPSA state and generate proposal
        match self.proposer.state().clone() {
            SpsaState::Ready => {
                // Start new iteration: generate +Δ perturbation
                let delta = self.proposer.generate_perturbation();
                self.proposer.start_plus_perturbation(delta.clone());
                
                Ok(Proposal::ApplyPlus {
                    perturbation_id: self.proposer.iteration(),
                    delta,
                })
            }
            SpsaState::WaitingPlus { perturbation_id, delta, .. } => {
                // Record objective value
                self.proposer.record_objective(digest.objective_value);
                
                // Check if we have enough samples
                if self.proposer.has_enough_samples() {
                    // Transition to minus phase
                    if let Some(_) = self.proposer.complete_eval_window() {
                        // This shouldn't return Some for plus window
                        unreachable!("Plus window shouldn't return gradient");
                    }
                    
                    // Generate −Δ perturbation
                    let minus_delta: ParamVec = delta.iter().map(|&d| -d).collect();
                    
                    Ok(Proposal::ApplyMinus {
                        perturbation_id,
                        delta: minus_delta,
                    })
                } else {
                    Ok(Proposal::NoChange { reason: NoChangeReason::EvalTimeout })
                }
            }
            SpsaState::WaitingMinus { perturbation_id: _, .. } => {
                // Record objective value
                self.proposer.record_objective(digest.objective_value);
                
                // Check if we have enough samples
                if self.proposer.has_enough_samples() {
                    // Complete both windows and compute gradient
                    if let Some((gradient, update_delta)) = self.proposer.complete_eval_window() {
                        Ok(Proposal::Update {
                            iteration: self.proposer.iteration() - 1,
                            delta: update_delta,
                            gradient_estimate: gradient,
                        })
                    } else {
                        Ok(Proposal::NoChange { reason: NoChangeReason::EvalTimeout })
                    }
                } else {
                    Ok(Proposal::NoChange { reason: NoChangeReason::EvalTimeout })
                }
            }
        }
    }

    /// Apply a proposal through safety guardrails.
    ///
    /// This is the Tier 1 apply path with audit logging.
    pub fn apply(&mut self, proposal: Proposal) -> Result<ApplyReceipt, Violation> {
        let now_us = Self::get_timestamp_us();
        let config_gen = self.config.generation();
        
        // Emit proposal audit event
        let audit_result = self.audit_queue.enqueue(
            AuditEvent::new(EventType::Proposal, now_us, self.run_id, config_gen)
                .with_proposal_id(self.proposer.iteration())
        );
        
        // Check audit queue pressure
        if audit_result == EnqueueResult::Full {
            return Err(Violation::AuditQueueFull);
        }
        
        // Apply through executor
        let result = self.executor.apply(proposal);
        
        // Emit apply audit event
        let _ = self.audit_queue.enqueue(
            AuditEvent::new(EventType::Apply, Self::get_timestamp_us(), self.run_id, self.config.generation())
                .with_proposal_id(self.proposer.iteration())
                .with_payload(if result.is_ok() { "success" } else { "violation" })
        );
        
        result
    }

    /// Rollback to baseline configuration.
    ///
    /// Emits rollback audit event.
    pub fn rollback(&mut self) -> Result<RollbackReceipt, Violation> {
        let now_us = Self::get_timestamp_us();
        let config_gen = self.config.generation();
        
        // Emit rollback audit event
        let _ = self.audit_queue.enqueue(
            AuditEvent::new(EventType::Rollback, now_us, self.run_id, config_gen)
        );
        
        self.executor.rollback()
    }

    /// Set current config as baseline for future rollbacks.
    pub fn set_baseline(&mut self) {
        self.executor.set_baseline();
    }

    /// Get current configuration snapshot (zero-copy via Arc).
    pub fn current(&self) -> ConfigSnapshot {
        self.executor.snapshot()
    }
    
    /// Get the audit queue for draining events.
    pub fn audit_queue(&self) -> &AuditQueue {
        &self.audit_queue
    }
    
    /// Get current SPSA iteration.
    pub fn iteration(&self) -> u64 {
        self.proposer.iteration()
    }
    
    /// Get telemetry buffer drop count.
    pub fn telemetry_drop_count(&self) -> u64 {
        self.telemetry_buffer.drop_count()
    }
    
    fn get_timestamp_us() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_engine_new() {
        let config = AdaptiveEngineConfig::default();
        let params = config_atomic::param_vec(&[0.5, 0.3, 0.2]);
        
        let engine = AdaptiveEngine::new(config, params);
        assert_eq!(engine.iteration(), 0);
        assert_eq!(engine.current().generation, 0);
    }

    #[test]
    fn test_adaptive_engine_set_baseline() {
        let config = AdaptiveEngineConfig::default();
        let params = config_atomic::param_vec(&[0.5]);
        
        let mut engine = AdaptiveEngine::new(config, params);
        engine.set_baseline();
        
        // Apply a change
        let delta = config_atomic::param_vec(&[0.05]);
        let proposal = Proposal::Update {
            iteration: 0,
            delta,
            gradient_estimate: config_atomic::param_vec(&[1.0]),
        };
        
        let _ = engine.apply(proposal);
        assert!((engine.current().params[0] - 0.55).abs() < 0.01);
        
        // Rollback
        let receipt = engine.rollback().unwrap();
        assert!((engine.current().params[0] - 0.5).abs() < 0.01);
        assert!(receipt.reverted_to_generation > 0);
    }

    #[test]
    fn test_adaptive_engine_observe_starts_spsa() {
        let config = AdaptiveEngineConfig::default();
        let params = config_atomic::param_vec(&[0.5]);
        
        let mut engine = AdaptiveEngine::new(config, params);
        
        // First observe should generate ApplyPlus
        let digest = TelemetryDigest::new(1000, 0.5, 0);
        let proposal = engine.observe(digest).unwrap();
        
        match proposal {
            Proposal::ApplyPlus { perturbation_id, delta } => {
                assert_eq!(perturbation_id, 0);
                assert_eq!(delta.len(), 1);
            }
            _ => panic!("Expected ApplyPlus, got {:?}", proposal),
        }
    }
}

