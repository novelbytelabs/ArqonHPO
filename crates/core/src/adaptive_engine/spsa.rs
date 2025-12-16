//! SPSA (Simultaneous Perturbation Stochastic Approximation) optimizer
//!
//! SPSA is ideal for online optimization because:
//! - Only 2 evaluations per update (regardless of dimension)
//! - Works well under noise
//! - Extremely cheap compute
//! - Deterministic with seed control

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Bernoulli};
use std::collections::HashMap;
use crate::config::Domain;

/// SPSA optimizer state
#[derive(Debug)]
pub struct Spsa {
    rng: ChaCha8Rng,
    bounds: HashMap<String, Domain>,
    /// Learning rate schedule parameter (a)
    a: f64,
    /// Perturbation scale (c)
    c: f64,
    /// Iteration counter
    k: u64,
    /// Stability constant for learning rate decay
    big_a: f64,
    /// Learning rate decay exponent
    alpha: f64,
    /// Perturbation decay exponent
    gamma: f64,
    /// State machine for two-measurement cycle
    state: SpsaState,
}

#[derive(Debug, Clone)]
enum SpsaState {
    /// Ready to propose a perturbation pair
    Ready,
    /// Waiting for first (y+) evaluation
    WaitingPlus { perturbation: HashMap<String, f64> },
    /// Waiting for second (y-) evaluation  
    WaitingMinus { 
        perturbation: HashMap<String, f64>,
        y_plus: f64,
    },
}

impl Spsa {
    /// Create a new SPSA optimizer
    pub fn new(
        seed: u64,
        learning_rate: f64,
        perturbation_scale: f64,
        bounds: HashMap<String, Domain>,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            bounds,
            a: learning_rate,
            c: perturbation_scale,
            k: 0,
            big_a: 100.0,   // Stability constant
            alpha: 0.602,   // Standard SPSA values
            gamma: 0.101,
            state: SpsaState::Ready,
        }
    }
    
    /// Current learning rate (decays with iteration)
    fn ak(&self) -> f64 {
        self.a / (self.k as f64 + 1.0 + self.big_a).powf(self.alpha)
    }
    
    /// Current perturbation magnitude (decays with iteration)
    fn ck(&self) -> f64 {
        self.c / (self.k as f64 + 1.0).powf(self.gamma)
    }
    
    /// Generate a random perturbation vector (±1 Bernoulli)
    fn generate_perturbation(&mut self) -> HashMap<String, f64> {
        let bernoulli = Bernoulli::new(0.5).unwrap();
        let ck = self.ck();
        
        self.bounds
            .keys()
            .map(|k| {
                let sign = if bernoulli.sample(&mut self.rng) { 1.0 } else { -1.0 };
                (k.clone(), sign * ck)
            })
            .collect()
    }
    
    /// Step the optimizer with a new objective value
    ///
    /// Returns `Some(delta)` when a full update is computed (after 2 evaluations),
    /// or `None` if still waiting for more data.
    ///
    /// # SPSA Algorithm
    /// 1. Generate random perturbation Δ (±1 Bernoulli scaled by c_k)
    /// 2. Evaluate at θ + Δ → y+
    /// 3. Evaluate at θ - Δ → y-
    /// 4. Compute gradient estimate: g = (y+ - y-) / (2 * Δ)
    /// 5. Update: θ_new = θ - a_k * g
    pub fn step(&mut self, objective_value: f64) -> Option<HashMap<String, f64>> {
        match &self.state {
            SpsaState::Ready => {
                // Generate new perturbation
                let perturbation = self.generate_perturbation();
                // First evaluation is at θ + Δ (caller should have applied this)
                self.state = SpsaState::WaitingPlus {
                    perturbation,
                };
                None
            }
            SpsaState::WaitingPlus { perturbation } => {
                // Got y+ evaluation, now need y- 
                self.state = SpsaState::WaitingMinus {
                    perturbation: perturbation.clone(),
                    y_plus: objective_value,
                };
                None
            }
            SpsaState::WaitingMinus { perturbation, y_plus } => {
                // Got y- evaluation, compute gradient and update
                let y_minus = objective_value;
                let ak = self.ak();
                
                // Compute update: θ_new = θ - a_k * (y+ - y-) / (2 * Δ)
                let delta: HashMap<String, f64> = perturbation
                    .iter()
                    .map(|(k, delta_k)| {
                        if delta_k.abs() < 1e-10 {
                            (k.clone(), 0.0)
                        } else {
                            // Gradient estimate for this dimension
                            let grad_k = (y_plus - y_minus) / (2.0 * delta_k);
                            // Update direction (negate for minimization)
                            (k.clone(), -ak * grad_k)
                        }
                    })
                    .collect();
                
                self.k += 1;
                self.state = SpsaState::Ready;
                
                Some(delta)
            }
        }
    }
    
    /// Get current iteration count
    pub fn iteration(&self) -> u64 {
        self.k
    }
    
    /// Get the current perturbation to apply (for plus measurement)
    pub fn current_perturbation_plus(&self) -> Option<HashMap<String, f64>> {
        match &self.state {
            SpsaState::WaitingPlus { perturbation, .. } => Some(perturbation.clone()),
            _ => None,
        }
    }
    
    /// Get the current perturbation to apply (for minus measurement)
    pub fn current_perturbation_minus(&self) -> Option<HashMap<String, f64>> {
        match &self.state {
            SpsaState::WaitingMinus { perturbation, .. } => {
                Some(perturbation.iter().map(|(k, v)| (k.clone(), -v)).collect())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spsa_deterministic() {
        let bounds = HashMap::from([
            ("x".to_string(), Domain { min: 0.0, max: 1.0, scale: crate::config::Scale::Linear }),
            ("y".to_string(), Domain { min: 0.0, max: 1.0, scale: crate::config::Scale::Linear }),
        ]);
        
        let mut spsa1 = Spsa::new(42, 0.1, 0.01, bounds.clone());
        let mut spsa2 = Spsa::new(42, 0.1, 0.01, bounds);
        
        // Both should generate same perturbations
        let p1 = spsa1.generate_perturbation();
        let p2 = spsa2.generate_perturbation();
        
        assert_eq!(p1, p2);
    }
    
    #[test]
    fn test_spsa_two_step_cycle() {
        let bounds = HashMap::from([
            ("x".to_string(), Domain { min: 0.0, max: 1.0, scale: crate::config::Scale::Linear }),
        ]);
        
        let mut spsa = Spsa::new(42, 0.1, 0.01, bounds);
        
        // First call: nothing (just sets up)
        assert!(spsa.step(0.0).is_none());
        
        // Provide y+ measurement
        assert!(spsa.step(1.0).is_none());
        
        // Provide y- measurement -> get delta
        let delta = spsa.step(0.5);
        assert!(delta.is_some());
        assert!(delta.unwrap().contains_key("x"));
    }
}
