//! Adaptive Engine for Online Optimization
//!
//! This module provides real-time parameter adaptation inside control loops.
//! Unlike the batch `Solver`, the `AdaptiveEngine` runs continuously, making
//! small adjustments based on streaming telemetry.
//!
//! # Architecture
//!
//! ```text
//! Telemetry → [AdaptiveEngine] → Proposals → [SafetyExecutor] → Config Swap
//! ```
//!
//! # Key Components
//!
//! - [`Spsa`]: SPSA optimizer for continuous knobs (2 evals per update)
//! - [`AtomicConfig`]: Lock-free configuration with atomic swap
//! - [`SafetyExecutor`]: Guardrails, bounds, rate limits, rollback
//! - [`TelemetryDigest`]: Compact telemetry summaries

mod spsa;
mod config_atomic;
mod safety_executor;
mod telemetry;

pub use spsa::Spsa;
pub use config_atomic::{AtomicConfig, ConfigSnapshot};
pub use safety_executor::{SafetyExecutor, Guardrails, Violation};
pub use telemetry::{TelemetryDigest, TelemetryRingBuffer};

use std::collections::HashMap;
use crate::config::Domain;

/// Configuration for the adaptive engine
#[derive(Debug, Clone)]
pub struct AdaptiveEngineConfig {
    /// Seed for deterministic PRNG
    pub seed: u64,
    /// Parameter bounds
    pub bounds: HashMap<String, Domain>,
    /// SPSA learning rate (a)
    pub learning_rate: f64,
    /// SPSA perturbation magnitude (c)
    pub perturbation_scale: f64,
    /// Maximum compute budget per decision cycle (microseconds)
    pub budget_us: u64,
    /// Guardrails configuration
    pub guardrails: Guardrails,
}

impl Default for AdaptiveEngineConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            bounds: HashMap::new(),
            learning_rate: 0.1,
            perturbation_scale: 0.01,
            budget_us: 1000, // 1ms default
            guardrails: Guardrails::default(),
        }
    }
}

/// The adaptive engine for online parameter tuning
///
/// Unlike the batch `Solver`, this runs in a tight loop and makes
/// incremental adjustments based on streaming objective values.
pub struct AdaptiveEngine {
    spsa: Spsa,
    safety: SafetyExecutor,
    current_config: AtomicConfig,
    telemetry: TelemetryRingBuffer,
}

impl AdaptiveEngine {
    /// Create a new adaptive engine
    pub fn new(config: AdaptiveEngineConfig, initial_params: HashMap<String, f64>) -> Self {
        let spsa = Spsa::new(
            config.seed,
            config.learning_rate,
            config.perturbation_scale,
            config.bounds.clone(),
        );
        
        let safety = SafetyExecutor::new(config.guardrails, config.bounds.clone());
        let current_config = AtomicConfig::new(initial_params);
        let telemetry = TelemetryRingBuffer::new(1024); // 1KB default
        
        Self {
            spsa,
            safety,
            current_config,
            telemetry,
        }
    }
    
    /// Get current configuration snapshot (zero-copy)
    pub fn current(&self) -> ConfigSnapshot {
        self.current_config.snapshot()
    }
    
    /// Submit a telemetry digest and get a proposed update
    ///
    /// Returns `Some(delta)` if an update is proposed, `None` if no change needed.
    /// The caller should apply via `apply_delta` after validation.
    pub fn observe(&mut self, digest: TelemetryDigest) -> Option<HashMap<String, f64>> {
        self.telemetry.push(digest.clone());
        
        // SPSA needs pairs of evaluations
        let delta = self.spsa.step(digest.objective_value)?;
        
        // Safety check
        if self.safety.validate_delta(&self.current_config.snapshot().params, &delta).is_ok() {
            Some(delta)
        } else {
            None
        }
    }
    
    /// Apply a delta to the current config
    ///
    /// The delta is validated through guardrails before applying.
    pub fn apply_delta(&mut self, delta: &HashMap<String, f64>) -> Result<(), Violation> {
        let current = self.current_config.snapshot();
        
        // Validate through safety executor
        self.safety.validate_delta(&current.params, delta)?;
        
        // Compute new params
        let mut new_params = current.params.clone();
        for (k, d) in delta {
            if let Some(v) = new_params.get_mut(k) {
                *v += d;
            }
        }
        
        // Apply bounds
        self.safety.clamp_to_bounds(&mut new_params);
        
        // Atomic swap
        self.current_config.swap(new_params);
        
        Ok(())
    }
    
    /// Rollback to baseline configuration
    pub fn rollback(&mut self) {
        if let Some(baseline) = self.safety.baseline() {
            self.current_config.swap(baseline);
        }
    }
    
    /// Set baseline for rollback
    pub fn set_baseline(&mut self) {
        self.safety.set_baseline(self.current_config.snapshot().params.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_engine_creation() {
        let config = AdaptiveEngineConfig::default();
        let params = HashMap::from([
            ("diffusion_factor".to_string(), 0.1),
            ("noise_level".to_string(), 0.01),
        ]);
        
        let engine = AdaptiveEngine::new(config, params.clone());
        let snapshot = engine.current();
        
        assert_eq!(snapshot.params, params);
    }
}
