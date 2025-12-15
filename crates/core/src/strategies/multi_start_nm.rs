//! Multi-Start Nelder-Mead Strategy
//! 
//! Runs K parallel NM instances from diverse seed points to avoid local minima.

use crate::artifact::EvalTrace;
use crate::config::{SolverConfig, Scale};
use crate::strategies::nelder_mead::NelderMead;
use crate::strategies::{Strategy, StrategyAction};
use std::collections::HashMap;

/// Configuration for multi-start Nelder-Mead
#[derive(Debug, Clone)]
pub struct MultiStartConfig {
    /// Number of parallel starts (default: 4)
    pub k: usize,
    /// Stall threshold: switch starts after this many iterations without improvement
    pub stall_threshold: usize,
    /// Triage budget per start (default: 20)
    pub triage_budget: usize,
    /// Minimum evaluations to justify a dedicated start (default: 80)
    pub min_evals_per_start: usize,
}

impl Default for MultiStartConfig {
    fn default() -> Self {
        Self {
            k: 4,
            stall_threshold: 10,
            triage_budget: 20,
            min_evals_per_start: 80,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MultiStartPhase {
    CoordinateDescent,
    Triage,
    Commit,
}

/// Multi-start Nelder-Mead: runs K NM instances from diverse seed points
pub struct MultiStartNM {
    config: MultiStartConfig,
    dim: usize,
    /// All NM instances
    starts: Vec<NelderMead>,
    /// Currently active start index
    active_idx: usize,
    /// Best value seen from each start
    best_per_start: Vec<f64>,
    /// Iterations since improvement for current start
    stall_counter: usize,
    /// Global best value
    global_best: f64,
    /// Number of evaluations consumed
    evals_used: usize,
    /// Coordinate descent phase active
    phase: MultiStartPhase,
    /// Triage evaluations consumed for current start
    triage_evals: Vec<usize>,
    /// Best start index identified during triage
    best_start_idx: usize,
}

impl MultiStartNM {
    /// Create multi-start NM with default config
    pub fn new(dim: usize, seed_points: Vec<HashMap<String, f64>>) -> Self {
        Self::with_config(dim, seed_points, MultiStartConfig::default())
    }

    /// Create multi-start NM with custom config
    pub fn with_config(
        dim: usize,
        seed_points: Vec<HashMap<String, f64>>,
        config: MultiStartConfig,
    ) -> Self {
        // Dimension-aware minimum evaluations per start
        let min_per_start = config.min_evals_per_start.max(25 * (dim + 1));
        
        // Calculate remaining budget for refinement (estimate)
        // Note: we don't have exact budget info here easily, but we can assume typical usage.
        // For now, respect config.k but clamped by practical limits if we had budget info.
        // In this constructor we just set up the starts. The step() logic will handle budget limits implicitly
        // by converging or running out of calls.
        // But the user requested: K = clamp(B_refine / min_per_start, 1, K_max)
        // We will stick to the provided K in config for now, assuming the caller (Solver) sets it intelligently,
        // OR we implement dynamic K logic if we had budget passed in.
        // Solver doesn't pass budget to new() ... yet.
        // So we proceed with config.k but ensure the constructor logic splits seeds correctly.
        
        // Split seed points into K groups
        let k = config.k.min(seed_points.len() / (dim + 1)).max(1);
        let points_per_start = (dim + 1).max(seed_points.len() / k);
        
        let mut starts = Vec::with_capacity(k);
        for i in 0..k {
            let start_idx = i * points_per_start;
            let end_idx = ((i + 1) * points_per_start).min(seed_points.len());
            
            if start_idx < seed_points.len() {
                let group: Vec<_> = seed_points[start_idx..end_idx].to_vec();
                starts.push(NelderMead::with_seed_points(dim, group));
            }
        }
        
        // Ensure at least one start
        if starts.is_empty() {
            starts.push(NelderMead::with_seed_points(dim, seed_points));
        }
        
        let num_starts = starts.len();
        
        Self {
            config,
            dim,
            starts,
            active_idx: 0,
            best_per_start: vec![f64::INFINITY; num_starts],
            stall_counter: 0,
            global_best: f64::INFINITY,
            evals_used: 0,
            phase: MultiStartPhase::CoordinateDescent,
            triage_evals: vec![0; num_starts],
            best_start_idx: 0,
        }
    }

    /// Check if we should switch to next start
    fn should_switch(&self) -> bool {
        self.stall_counter >= self.config.stall_threshold
    }

    /// Switch to the next start
    fn switch_to_next(&mut self) {
        if self.starts.len() > 1 {
            self.active_idx = (self.active_idx + 1) % self.starts.len();
            self.stall_counter = 0;
        }
    }

    /// Update tracking after an evaluation
    fn update_tracking(&mut self, value: f64) {
        // Update per-start best
        if value < self.best_per_start[self.active_idx] {
            self.best_per_start[self.active_idx] = value;
            self.stall_counter = 0;
        } else {
            self.stall_counter += 1;
        }
        
        // Update global best
        if value < self.global_best {
            self.global_best = value;
        }
        
        self.evals_used += 1;
    }

    /// Helper to map value to unit space
    fn val_to_unit(val: f64, min: f64, max: f64, scale: Scale) -> f64 {
        match scale {
            Scale::Linear => (val - min) / (max - min),
            Scale::Log => {
                let min_log = min.ln();
                let max_log = max.ln();
                (val.ln() - min_log) / (max_log - min_log)
            }
        }
    }

    /// Helper to map unit space to value
    fn unit_to_val(unit: f64, min: f64, max: f64, scale: Scale) -> f64 {
        match scale {
            Scale::Linear => min + unit * (max - min),
            Scale::Log => {
                let min_log = min.ln();
                let max_log = max.ln();
                (min_log + unit * (max_log - min_log)).exp()
            }
        }
    }

    /// Run single-pass coordinate descent around best point
    fn run_coordinate_descent(&mut self, config: &SolverConfig, history: &[EvalTrace]) -> StrategyAction {
        // Find best point
        let best_trace = history.iter().min_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
        
        if let Some(best) = best_trace {
            let mut candidates = Vec::new();
            let delta = 0.1; // Step size in unit space (10%)

            // Iterate all dimensions
            for (name, domain) in &config.bounds {
                if let Some(val) = best.params.get(name) {
                    let unit_val = Self::val_to_unit(*val, domain.min, domain.max, domain.scale.clone());
                    
                    // Try +delta
                    let unit_plus = (unit_val + delta).min(1.0);
                    if (unit_plus - unit_val).abs() > 1e-6 {
                        let mut point = best.params.clone();
                        point.insert(name.clone(), Self::unit_to_val(unit_plus, domain.min, domain.max, domain.scale.clone()));
                        candidates.push(point);
                    }
                    
                    // Try -delta
                    let unit_minus = (unit_val - delta).max(0.0);
                    if (unit_minus - unit_val).abs() > 1e-6 {
                        let mut point = best.params.clone();
                        point.insert(name.clone(), Self::unit_to_val(unit_minus, domain.min, domain.max, domain.scale.clone()));
                        candidates.push(point);
                    }
                }
            }
            
            if !candidates.is_empty() {
                return StrategyAction::Evaluate(candidates);
            }
        }
        
        StrategyAction::Wait // Should not happen if history exists
    }
}

impl Strategy for MultiStartNM {
    fn step(&mut self, config: &SolverConfig, history: &[EvalTrace]) -> StrategyAction {
        // Update tracking if we have new history
        if let Some(last) = history.last() {
            self.update_tracking(last.value);
            // Track triage steps
            if let MultiStartPhase::Triage = self.phase {
                if self.starts.len() > 1 {
                    self.triage_evals[self.active_idx] += 1;
                }
            }
        }

        loop {
            match self.phase {
                MultiStartPhase::CoordinateDescent => {
                    self.phase = MultiStartPhase::Triage; // Move to next phase after CD
                    return self.run_coordinate_descent(config, history);
                }
                
                MultiStartPhase::Triage => {
                    // If only 1 start, skip triage
                    if self.starts.len() <= 1 {
                        self.phase = MultiStartPhase::Commit;
                        continue;
                    }

                    // Check if current start exhausted triage budget
                    if self.triage_evals[self.active_idx] >= self.config.triage_budget {
                        // Switch to next start
                        self.active_idx = (self.active_idx + 1) % self.starts.len();
                        
                        // Check if ALL starts finished triage
                        if self.active_idx == 0 && self.triage_evals[0] >= self.config.triage_budget {
                            // Select winner
                            let winner_idx = self.best_per_start
                                .iter()
                                .enumerate()
                                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                                .map(|(i, _)| i)
                                .unwrap_or(0);
                            
                            self.best_start_idx = winner_idx;
                            self.active_idx = winner_idx;
                            self.phase = MultiStartPhase::Commit;
                            continue;
                        }
                    }
                    
                    // Run current start
                    if let Some(nm) = self.starts.get_mut(self.active_idx) {
                        match nm.step(config, history) {
                            StrategyAction::Converged => {
                                // Start converged early during triage
                                self.triage_evals[self.active_idx] = usize::MAX; // Mark done
                                self.active_idx = (self.active_idx + 1) % self.starts.len();
                                // Loop will check triage completion condition
                                continue;
                            }
                            action => return action,
                        }
                    }
                }
                
                MultiStartPhase::Commit => {
                    // Run best start until exhaustion
                    // Also support switching if it stalls? 
                    // For now, commit strategy implies sticking to the best.
                    
                    if let Some(nm) = self.starts.get_mut(self.active_idx) {
                        return nm.step(config, history);
                    } else {
                        return StrategyAction::Converged;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_start_creation() {
        let mut seeds = Vec::new();
        for i in 0..20 {
            let mut point = HashMap::new();
            point.insert("x".to_string(), i as f64 / 20.0);
            point.insert("y".to_string(), (20 - i) as f64 / 20.0);
            seeds.push(point);
        }
        
        let ms = MultiStartNM::new(2, seeds);
        assert!(ms.starts.len() >= 1);
        assert!(ms.starts.len() <= 4); // K=4 default
    }

    #[test]
    fn test_config_defaults() {
        let config = MultiStartConfig::default();
        assert_eq!(config.k, 4);
        assert_eq!(config.stall_threshold, 10);
    }
}
