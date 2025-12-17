//! SPSA (Simultaneous Perturbation Stochastic Approximation) optimizer.
//!
//! Constitution: II.16 - SPSA MUST use ±1 Bernoulli perturbations, ChaCha8Rng,
//! and decay schedules α=0.602, γ=0.101.

use crate::config_atomic::ParamVec;
use rand::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// SPSA state machine states.
#[derive(Clone, Debug, PartialEq)]
pub enum SpsaState {
    /// Ready to start a new iteration.
    Ready,
    /// Applied +Δ, waiting to collect eval window.
    WaitingPlus {
        perturbation_id: u64,
        delta: ParamVec,
        accumulated: Vec<f64>,
    },
    /// Applied −Δ, waiting to collect eval window.
    WaitingMinus {
        perturbation_id: u64,
        delta: ParamVec,
        y_plus: f64,
        accumulated: Vec<f64>,
    },
}

/// SPSA configuration.
#[derive(Clone, Debug)]
pub struct SpsaConfig {
    /// Minimum digests to collect per perturbation.
    pub eval_window_digests: usize,
    /// Maximum time to wait for digests (microseconds).
    pub eval_window_us: u64,
    /// Time after apply before digests count (microseconds).
    pub settle_time_us: u64,
    /// Learning rate decay: a_k = a0 / (k + 1 + A)^α
    pub alpha: f64,
    /// Perturbation decay: c_k = c0 / (k + 1)^γ
    pub gamma: f64,
    /// Stability constant A.
    pub stability_a: f64,
}

impl Default for SpsaConfig {
    fn default() -> Self {
        Self {
            eval_window_digests: 5,
            eval_window_us: 500_000,
            settle_time_us: 10_000,
            alpha: 0.602,
            gamma: 0.101,
            stability_a: 10.0,
        }
    }
}

/// SPSA optimizer (Tier 2 component).
///
/// Constitution: II.16 - SPSA Specification
pub struct Spsa {
    rng: ChaCha8Rng,
    state: SpsaState,
    iteration: u64,
    perturbation_counter: u64,
    config: SpsaConfig,
    initial_learning_rate: f64,
    initial_perturbation_scale: f64,
    num_params: usize,
}

impl Spsa {
    /// Create a new SPSA optimizer.
    pub fn new(
        seed: u64,
        num_params: usize,
        learning_rate: f64,
        perturbation_scale: f64,
        config: SpsaConfig,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            state: SpsaState::Ready,
            iteration: 0,
            perturbation_counter: 0,
            config,
            initial_learning_rate: learning_rate,
            initial_perturbation_scale: perturbation_scale,
            num_params,
        }
    }

    /// Get current iteration count.
    pub fn iteration(&self) -> u64 {
        self.iteration
    }

    /// Get current perturbation counter.
    pub fn perturbation_counter(&self) -> u64 {
        self.perturbation_counter
    }

    /// Get current state.
    pub fn state(&self) -> &SpsaState {
        &self.state
    }

    /// Compute learning rate for iteration k.
    pub fn learning_rate(&self, k: u64) -> f64 {
        let k_f = k as f64;
        self.initial_learning_rate / (k_f + 1.0 + self.config.stability_a).powf(self.config.alpha)
    }

    /// Compute perturbation scale for iteration k.
    pub fn perturbation_scale(&self, k: u64) -> f64 {
        let k_f = k as f64;
        self.initial_perturbation_scale / (k_f + 1.0).powf(self.config.gamma)
    }

    /// Generate a perturbation vector using ±1 Bernoulli distribution.
    pub fn generate_perturbation(&mut self) -> ParamVec {
        let c_k = self.perturbation_scale(self.iteration);
        let mut delta = ParamVec::with_capacity(self.num_params);

        for _ in 0..self.num_params {
            let sign = if self.rng.random::<bool>() { 1.0 } else { -1.0 };
            delta.push(sign * c_k);
        }

        self.perturbation_counter += 1;
        delta
    }

    /// Signal that we're starting to apply +Δ.
    pub fn start_plus_perturbation(&mut self, delta: ParamVec) {
        self.state = SpsaState::WaitingPlus {
            perturbation_id: self.perturbation_counter,
            delta,
            accumulated: Vec::new(),
        };
    }

    /// Signal that we're starting to apply −Δ.
    pub fn start_minus_perturbation(&mut self, delta: ParamVec, y_plus: f64) {
        self.state = SpsaState::WaitingMinus {
            perturbation_id: self.perturbation_counter,
            delta,
            y_plus,
            accumulated: Vec::new(),
        };
    }

    /// Record an objective value from the eval window.
    pub fn record_objective(&mut self, value: f64) {
        match &mut self.state {
            SpsaState::WaitingPlus { accumulated, .. }
            | SpsaState::WaitingMinus { accumulated, .. } => {
                accumulated.push(value);
            }
            SpsaState::Ready => {}
        }
    }

    /// Check if we have enough samples in the current eval window.
    pub fn has_enough_samples(&self) -> bool {
        match &self.state {
            SpsaState::WaitingPlus { accumulated, .. }
            | SpsaState::WaitingMinus { accumulated, .. } => {
                accumulated.len() >= self.config.eval_window_digests
            }
            SpsaState::Ready => false,
        }
    }

    /// Aggregate objective values using trimmed mean.
    pub fn aggregate_objectives(values: &[f64], trim_percent: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        if values.len() == 1 {
            return values[0];
        }

        let mut sorted: Vec<f64> = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let trim_count = ((values.len() as f64) * trim_percent).ceil() as usize;
        let trimmed = &sorted[trim_count..sorted.len().saturating_sub(trim_count)];

        if trimmed.is_empty() {
            sorted.iter().sum::<f64>() / sorted.len() as f64
        } else {
            trimmed.iter().sum::<f64>() / trimmed.len() as f64
        }
    }

    /// Complete the current eval window and compute gradient/update.
    ///
    /// Returns Some((gradient, update_delta)) if both windows completed,
    /// None if still waiting for minus window.
    pub fn complete_eval_window(&mut self) -> Option<(ParamVec, ParamVec)> {
        match std::mem::replace(&mut self.state, SpsaState::Ready) {
            SpsaState::WaitingPlus {
                delta, accumulated, ..
            } => {
                let y_plus = Self::aggregate_objectives(&accumulated, 0.1);

                // Transition to minus phase
                let _minus_delta: ParamVec = delta.iter().map(|&d| -d).collect();
                self.perturbation_counter += 1;
                self.state = SpsaState::WaitingMinus {
                    perturbation_id: self.perturbation_counter,
                    delta,
                    y_plus,
                    accumulated: Vec::new(),
                };
                None
            }
            SpsaState::WaitingMinus {
                delta,
                y_plus,
                accumulated,
                ..
            } => {
                let y_minus = Self::aggregate_objectives(&accumulated, 0.1);

                // Compute gradient: g_k = (y+ - y-) / (2 * Δ)
                let a_k = self.learning_rate(self.iteration);
                let mut gradient = ParamVec::with_capacity(self.num_params);
                let mut update_delta = ParamVec::with_capacity(self.num_params);

                for &d in delta.iter() {
                    let g = (y_plus - y_minus) / (2.0 * d);
                    gradient.push(g);
                    update_delta.push(-a_k * g);
                }

                self.iteration += 1;
                self.state = SpsaState::Ready;

                Some((gradient, update_delta))
            }
            SpsaState::Ready => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spsa_deterministic() {
        let mut spsa1 = Spsa::new(42, 3, 0.1, 0.01, SpsaConfig::default());
        let mut spsa2 = Spsa::new(42, 3, 0.1, 0.01, SpsaConfig::default());

        let delta1 = spsa1.generate_perturbation();
        let delta2 = spsa2.generate_perturbation();

        assert_eq!(delta1.as_slice(), delta2.as_slice());
    }

    #[test]
    fn test_spsa_perturbation_signs() {
        let mut spsa = Spsa::new(123, 5, 0.1, 1.0, SpsaConfig::default());
        let delta = spsa.generate_perturbation();

        // All values should be ±1.0 (since perturbation_scale starts at 1.0)
        for &d in delta.iter() {
            assert!((d.abs() - 1.0).abs() < 1e-10, "Expected ±1.0, got {}", d);
        }
    }

    #[test]
    fn test_learning_rate_decay() {
        let spsa = Spsa::new(0, 1, 0.1, 0.01, SpsaConfig::default());

        let a0 = spsa.learning_rate(0);
        let a10 = spsa.learning_rate(10);
        let a100 = spsa.learning_rate(100);

        assert!(a0 > a10);
        assert!(a10 > a100);
    }

    #[test]
    fn test_trimmed_mean() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        let result = Spsa::aggregate_objectives(&values, 0.2);

        // With 20% trim on 5 values, removes 1 from each end
        // Should average [2.0, 3.0, 4.0] = 3.0
        assert!((result - 3.0).abs() < 1e-10);
    }
}
