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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::TelemetryDigest;

    fn default_digest(objective_value: f64) -> TelemetryDigest {
        TelemetryDigest::new(1000, objective_value, 1)
    }

    #[test]
    fn test_adaptive_engine_config_default() {
        let config = AdaptiveEngineConfig::default();
        assert_eq!(config.seed, 42);
        assert_eq!(config.learning_rate, 0.1);
        assert_eq!(config.perturbation_scale, 0.01);
    }

    #[test]
    fn test_spsa_proposer_new() {
        let spsa = Spsa::new(42, 2, 0.1, 0.01, SpsaConfig::default());
        let proposer = SpsaProposer::new(spsa);
        assert!(proposer.current_perturbation().is_none());
        assert_eq!(proposer.iteration(), 0);
    }

    #[test]
    fn test_spsa_proposer_observe_ready_state() {
        let spsa = Spsa::new(42, 2, 0.1, 0.01, SpsaConfig::default());
        let mut proposer = SpsaProposer::new(spsa);

        let digest = default_digest(1.0);
        let result = proposer.observe(digest);

        assert!(result.is_ok());
        match result.unwrap() {
            Proposal::ApplyPlus {
                perturbation_id,
                delta,
            } => {
                assert!(perturbation_id > 0);
                assert_eq!(delta.len(), 2);
            }
            _ => panic!("Expected ApplyPlus proposal"),
        }
    }

    #[test]
    fn test_spsa_proposer_state_inspection() {
        let spsa = Spsa::new(42, 2, 0.1, 0.01, SpsaConfig::default());
        let proposer = SpsaProposer::new(spsa);

        // Initial state should be Ready
        match proposer.spsa_state() {
            SpsaState::Ready => {}
            _ => panic!("Expected Ready state"),
        }
    }

    #[test]
    fn test_adaptive_engine_new() {
        let config = AdaptiveEngineConfig::default();
        let initial_params = ParamVec::from_slice(&[0.5, 0.5]);
        let engine = AdaptiveEngine::new(config, initial_params);

        let snapshot = engine.snapshot();
        assert_eq!(snapshot.params.len(), 2);
    }

    #[test]
    fn test_adaptive_engine_snapshot() {
        let config = AdaptiveEngineConfig::default();
        let initial_params = ParamVec::from_slice(&[0.3, 0.7]);
        let engine = AdaptiveEngine::new(config, initial_params);

        let snapshot = engine.snapshot();
        assert_eq!(snapshot.params[0], 0.3);
        assert_eq!(snapshot.params[1], 0.7);
    }

    #[test]
    fn test_adaptive_engine_observe() {
        let config = AdaptiveEngineConfig::default();
        let initial_params = ParamVec::from_slice(&[0.5, 0.5]);
        let mut engine = AdaptiveEngine::new(config, initial_params);

        let digest = default_digest(1.0);
        let result = engine.observe(digest);

        assert!(result.is_ok());
    }

    #[test]
    fn test_adaptive_engine_spsa_state() {
        let config = AdaptiveEngineConfig::default();
        let initial_params = ParamVec::from_slice(&[0.5, 0.5]);
        let engine = AdaptiveEngine::new(config, initial_params);

        // Should be in Ready state initially
        match engine.spsa_state() {
            SpsaState::Ready => {}
            _ => panic!("Expected Ready state"),
        }
    }

    #[test]
    fn test_spsa_proposer_current_perturbation_after_observe() {
        let spsa = Spsa::new(42, 2, 0.1, 0.01, SpsaConfig::default());
        let mut proposer = SpsaProposer::new(spsa);

        let digest = default_digest(1.0);
        let _ = proposer.observe(digest);

        // After observing in Ready state, should have a perturbation
        let perturbation = proposer.current_perturbation();
        assert!(perturbation.is_some());
        let (id, delta) = perturbation.unwrap();
        assert!(id > 0);
        assert_eq!(delta.len(), 2);
    }

    #[test]
    fn test_spsa_proposer_waiting_plus_no_samples() {
        let spsa = Spsa::new(42, 2, 0.1, 0.01, SpsaConfig::default());
        let mut proposer = SpsaProposer::new(spsa);

        // First observe transitions to WaitingPlus
        let _ = proposer.observe(default_digest(1.0));

        // Second observe in WaitingPlus without enough samples
        let result = proposer.observe(default_digest(0.9));
        assert!(result.is_ok());
        match result.unwrap() {
            Proposal::NoChange { reason } => {
                assert!(matches!(reason, NoChangeReason::EvalTimeout));
            }
            Proposal::ApplyMinus { .. } => {} // Also valid if samples reached
            _ => panic!("Expected NoChange or ApplyMinus"),
        }
    }

    #[test]
    fn test_adaptive_engine_apply() {
        let config = AdaptiveEngineConfig::default();
        let initial_params = ParamVec::from_slice(&[0.5, 0.5]);
        let mut engine = AdaptiveEngine::new(config, initial_params);

        // Create a simple proposal
        let proposal = Proposal::NoChange {
            reason: NoChangeReason::EvalTimeout,
        };
        let result = engine.apply(proposal);

        // NoChange should be applied successfully
        assert!(result.is_ok());
    }
}
