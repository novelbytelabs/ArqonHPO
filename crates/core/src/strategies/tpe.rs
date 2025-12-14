use crate::config::SolverConfig;
use crate::artifact::EvalTrace;
use crate::strategies::{Strategy, StrategyAction};
use crate::rng::get_rng;
use rand::Rng;
use std::collections::HashMap;
use rand_chacha::ChaCha8Rng;

#[allow(dead_code)]
pub struct TPE {
    dim: usize,
    gamma: f64, 
    candidates: usize, 
}

impl TPE {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            gamma: 0.25, // Top 25%
            candidates: 24,
        }
    }

    // Gaussian PDF
    fn pdf(x: f64, mean: f64, sigma: f64) -> f64 {
        let denom = (2.0 * std::f64::consts::PI).sqrt() * sigma;
        let num = (-0.5 * ((x - mean) / sigma).powi(2)).exp();
        num / denom
    }

    // Sample from GMM: Pick a component (point), then sample Gaussian.
    fn sample_gmm(rng: &mut ChaCha8Rng, points: &[f64], sigma: f64, min: f64, max: f64) -> f64 {
        if points.is_empty() {
            return rng.random_range(min..=max);
        }
        let idx = rng.random_range(0..points.len());
        let mean = points[idx];
        let val = mean + rng.sample::<f64, _>(rand_distr::StandardNormal) * sigma;
        val.clamp(min, max)
    }
}

impl Strategy for TPE {
    fn step(&mut self, config: &SolverConfig, history: &[EvalTrace]) -> StrategyAction {
        if history.len() < self.candidates {
             // Not enough data to build model, fallback to random
             use crate::probe::UniformProbe;
             use crate::probe::Probe;
             let p = UniformProbe;
             return StrategyAction::Evaluate(p.sample(config).into_iter().take(1).collect());
        }

        let mut rng = get_rng(config.seed + history.len() as u64);
        
        // 1. Sort by value
        let mut sorted: Vec<_> = history.iter().collect();
        sorted.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
        
        let split_idx = (history.len() as f64 * self.gamma).ceil() as usize;
        let split_idx = split_idx.max(2); // Min 2 good points
        let (good, bad) = sorted.split_at(split_idx);
        
        // For each param, build 1D GMM
        let mut best_candidate = HashMap::new();
        let mut best_ei = -1.0;
        
        // We generate N candidates and pick best EI
        // But here we do it dimension-wise assumption (independent params).
        // Actually, TPE usually samples a vector by sampling each dim independently from l(x).
        // Then computes EI for that vector.
        
        let mut candidates_vec = Vec::new();

        for _ in 0..self.candidates {
            let mut candidate = HashMap::new();
            let mut log_l = 0.0;
            let mut log_g = 0.0;
            
            for (name, domain) in &config.bounds {
                // Collect values for this dimension
                let good_vals: Vec<f64> = good.iter().map(|t| *t.params.get(name).unwrap_or(&0.0)).collect();
                let bad_vals: Vec<f64> = bad.iter().map(|t| *t.params.get(name).unwrap_or(&0.0)).collect();
                
                // Bandwidth: Rule of thumb sigma = (max - min) / sqrt(N)? 
                // Or standard deviation of data?
                // For MVP: Use 10% of range or stddev.
                let range = domain.max - domain.min;
                let sigma = range * 0.1; // simplified
                 
                // Sample from l(x) (Good)
                let val = Self::sample_gmm(&mut rng, &good_vals, sigma, domain.min, domain.max);
                candidate.insert(name.clone(), val);
                
                // Compute Likelihoods
                let l_prob: f64 = good_vals.iter().map(|&m| Self::pdf(val, m, sigma)).sum::<f64>() / good_vals.len() as f64;
                let g_prob: f64 = bad_vals.iter().map(|&m| Self::pdf(val, m, sigma)).sum::<f64>() / bad_vals.len() as f64;
                
                // Avoid log(0)
                let l_prob = l_prob.max(1e-12);
                let g_prob = g_prob.max(1e-12);

                log_l += l_prob.ln();
                log_g += g_prob.ln();
            }
            
            // EI ~ l(x) / g(x) -> log EI ~ log l - log g
            let ei = log_l - log_g;
            candidates_vec.push(candidate.clone()); // push clone before move
            if ei > best_ei || best_candidate.is_empty() {
                best_ei = ei;
                best_candidate = candidate;
            }
        }
        
        // Return best of N candidates
        StrategyAction::Evaluate(vec![best_candidate])
    }
}
