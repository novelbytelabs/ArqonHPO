//! AdaptiveEngine: High-level orchestrator combining SPSA, Proposer, and Config.
//!
//! Constitution: II.16-23 - Tier 2 Adaptive Engine

use crate::{
    config_atomic::{AtomicConfig, ConfigSnapshot, ParamVec},
    executor::{ApplyReceipt, Guardrails, SafeExecutor, SafetyExecutor, Violation},
    proposer::{AdaptiveProposer, NoChangeReason, Proposal, ProposalError, ProposalResult},
    spsa::{Spsa, SpsaConfig, SpsaState},
    telemetry::TelemetryDigest,
};
use std::sync::Arc;

/// Configuration for AdaptiveEngine.
#[derive(Clone, Debug)]
pub struct AdaptiveEngineConfig {
    /// SPSA configuration.
    pub spsa: SpsaConfig,
    /// Guardrails for safety executor.
    pub guardrails: Guardrails,
    /// Seed for RNG.
    pub seed: u64,
    /// Initial learning rate.
    pub learning_rate: f64,
    /// Initial perturbation scale.
    pub perturbation_scale: f64,
}

impl Default for AdaptiveEngineConfig {
    fn default() -> Self {
        Self {
            spsa: SpsaConfig::default(),
            guardrails: Guardrails::default(),
            seed: 42,
            learning_rate: 0.1,
            perturbation_scale: 0.01,
        }
    }
}

/// Concrete SPSA-based proposer implementing AdaptiveProposer trait.
pub struct SpsaProposer {
    spsa: Spsa,
    current_delta: Option<ParamVec>,
}

impl SpsaProposer {
    /// Create a new SPSA proposer.
    pub fn new(spsa: Spsa) -> Self {
        Self {
            spsa,
            current_delta: None,
        }
    }

    /// Get SPSA state for inspection.
    pub fn spsa_state(&self) -> &SpsaState {
        self.spsa.state()
    }
}

impl AdaptiveProposer for SpsaProposer {
    fn observe(&mut self, digest: TelemetryDigest) -> ProposalResult {
        // Record objective value from digest
        self.spsa.record_objective(digest.objective_value);

        match self.spsa.state() {
            SpsaState::Ready => {
                // Generate new perturbation and start plus phase
                let delta = self.spsa.generate_perturbation();
                self.current_delta = Some(delta.clone());
                self.spsa.start_plus_perturbation(delta.clone());
                Ok(Proposal::ApplyPlus {
                    perturbation_id: self.spsa.perturbation_counter(),
                    delta,
                })
            }
            SpsaState::WaitingPlus { .. } => {
                // Check if we have enough samples
                if self.spsa.has_enough_samples() {
                    // Complete plus window, transition to minus
                    let _ = self.spsa.complete_eval_window();
                    // Apply minus delta
                    if let Some(ref delta) = self.current_delta {
                        let minus_delta: ParamVec = delta.iter().map(|&d| -d).collect();
                        Ok(Proposal::ApplyMinus {
                            perturbation_id: self.spsa.perturbation_counter(),
                            delta: minus_delta,
                        })
                    } else {
                        Err(ProposalError::InternalError(
                            "No delta available".to_string(),
                        ))
                    }
                } else {
                    Ok(Proposal::NoChange {
                        reason: NoChangeReason::EvalTimeout,
                    })
                }
            }
            SpsaState::WaitingMinus { .. } => {
                if self.spsa.has_enough_samples() {
                    // Complete minus window, compute update
                    if let Some((_gradient, update_delta)) = self.spsa.complete_eval_window() {
                        self.current_delta = None;
                        Ok(Proposal::Update {
                            iteration: self.spsa.iteration(),
                            delta: update_delta.clone(),
                            gradient_estimate: update_delta,
                        })
                    } else {
                        Ok(Proposal::NoChange {
                            reason: NoChangeReason::EvalTimeout,
                        })
                    }
                } else {
                    Ok(Proposal::NoChange {
                        reason: NoChangeReason::EvalTimeout,
                    })
                }
            }
        }
    }

    fn current_perturbation(&self) -> Option<(u64, ParamVec)> {
        self.current_delta
            .clone()
            .map(|d| (self.spsa.perturbation_counter(), d))
    }

    fn iteration(&self) -> u64 {
        self.spsa.iteration()
    }
}

/// High-level adaptive engine orchestrating SPSA, Proposer, and Executor.
pub struct AdaptiveEngine {
    proposer: SpsaProposer,
    config: Arc<AtomicConfig>,
    executor: SafetyExecutor,
}

impl AdaptiveEngine {
    /// Create a new AdaptiveEngine.
    pub fn new(engine_config: AdaptiveEngineConfig, initial_params: ParamVec) -> Self {
        let config = Arc::new(AtomicConfig::new(initial_params.clone()));
        let num_params = initial_params.len();

        let spsa = Spsa::new(
            engine_config.seed,
            num_params,
            engine_config.learning_rate,
            engine_config.perturbation_scale,
            engine_config.spsa.clone(),
        );

        let proposer = SpsaProposer::new(spsa);
        let executor = SafetyExecutor::new(config.clone(), engine_config.guardrails);

        Self {
            proposer,
            config,
            executor,
        }
    }

    /// Observe a telemetry digest and potentially get a proposal.
    pub fn observe(&mut self, digest: TelemetryDigest) -> ProposalResult {
        self.proposer.observe(digest)
    }

    /// Get current configuration snapshot.
    pub fn snapshot(&self) -> Arc<ConfigSnapshot> {
        self.config.snapshot()
    }

    /// Apply a proposal through the safety executor.
    pub fn apply(&mut self, proposal: Proposal) -> Result<ApplyReceipt, Violation> {
        self.executor.apply(proposal)
    }

    /// Get SPSA state for inspection.
    pub fn spsa_state(&self) -> &SpsaState {
        self.proposer.spsa_state()
    }
}
