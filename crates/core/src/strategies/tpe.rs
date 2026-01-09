use crate::artifact::EvalTrace;
use crate::config::SolverConfig;
use crate::rng::get_rng;
use crate::strategies::{Strategy, StrategyAction};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

/// Bandwidth selection rule for kernel density estimation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BandwidthRule {
    /// Scott's Rule: σ = 1.06 × stddev × n^(-1/5)
    #[default]
    Scott,
    /// Silverman's Rule: σ = 0.9 × min(stddev, IQR/1.34) × n^(-1/5)
    Silverman,
    /// Fixed percentage of range
    Fixed,
}

#[allow(dead_code)]
pub struct TPE {
    dim: usize,
    gamma: f64,
    candidates: usize,
    pub bandwidth_rule: BandwidthRule,
}

impl TPE {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            gamma: 0.25, // Top 25%
            candidates: 24,
            bandwidth_rule: BandwidthRule::Scott,
        }
    }

    /// Create TPE with a specific bandwidth rule
    pub fn with_bandwidth_rule(dim: usize, rule: BandwidthRule) -> Self {
        Self {
            dim,
            gamma: 0.25,
            candidates: 24,
            bandwidth_rule: rule,
        }
    }

    /// Scott's Rule bandwidth: σ = 1.06 × stddev × n^(-1/5)
    ///
    /// This is the standard bandwidth for kernel density estimation.
    /// It adapts to the sample distribution and count.
    pub fn scotts_bandwidth(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 1.0; // Fallback for insufficient data
        }

        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        // Scott's Rule: 1.06 × σ × n^(-1/5)
        let bandwidth = 1.06 * stddev * n.powf(-0.2);

        // Minimum bandwidth to prevent degenerate kernels
        bandwidth.max(1e-6)
    }

    /// Silverman's Rule bandwidth: σ = 0.9 × min(stddev, IQR/1.34) × n^(-1/5)
    ///
    /// More robust to outliers than Scott's Rule.
    #[allow(dead_code)]
    pub fn silverman_bandwidth(values: &[f64]) -> f64 {
        if values.len() < 4 {
            return Self::scotts_bandwidth(values); // Fall back to Scott's
        }

        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        // Compute IQR
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let q1_idx = values.len() / 4;
        let q3_idx = 3 * values.len() / 4;
        let iqr = sorted[q3_idx] - sorted[q1_idx];

        // Silverman's robust estimate
        let scale = stddev.min(iqr / 1.34);
        let bandwidth = 0.9 * scale * n.powf(-0.2);

        bandwidth.max(1e-6)
    }

    /// Fixed bandwidth: percentage of range (legacy behavior)
    #[allow(dead_code)]
    pub fn fixed_bandwidth(range: f64, percentage: f64) -> f64 {
        (range * percentage).max(1e-6)
    }

    /// Compute bandwidth for a dimension based on selected rule
    fn compute_bandwidth(&self, values: &[f64], range: f64) -> f64 {
        match self.bandwidth_rule {
            BandwidthRule::Scott => Self::scotts_bandwidth(values),
            BandwidthRule::Silverman => Self::silverman_bandwidth(values),
            BandwidthRule::Fixed => Self::fixed_bandwidth(range, 0.1),
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
            // Not enough data to build model, fallback to random sampling
            // Use history.len() as part of seed to ensure different samples on each call
            let mut rng = get_rng(config.seed + history.len() as u64);
            let mut candidate = HashMap::new();
            for (name, domain) in &config.bounds {
                let val = rng.random_range(domain.min..=domain.max);
                candidate.insert(name.clone(), val);
            }
            return StrategyAction::Evaluate(vec![candidate]);
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

        let mut candidates_vec = Vec::new();

        for _ in 0..self.candidates {
            let mut candidate = HashMap::new();
            let mut log_l = 0.0;
            let mut log_g = 0.0;

            for (name, domain) in &config.bounds {
                // Collect values for this dimension
                let good_vals: Vec<f64> = good
                    .iter()
                    .map(|t| *t.params.get(name).unwrap_or(&0.0))
                    .collect();
                let bad_vals: Vec<f64> = bad
                    .iter()
                    .map(|t| *t.params.get(name).unwrap_or(&0.0))
                    .collect();

                // Compute adaptive bandwidth using Scott's Rule (or selected rule)
                let range = domain.max - domain.min;
                let sigma = self.compute_bandwidth(&good_vals, range);

                // Sample from l(x) (Good)
                let val = Self::sample_gmm(&mut rng, &good_vals, sigma, domain.min, domain.max);
                candidate.insert(name.clone(), val);

                // Compute Likelihoods
                let l_prob: f64 = good_vals
                    .iter()
                    .map(|&m| Self::pdf(val, m, sigma))
                    .sum::<f64>()
                    / good_vals.len() as f64;
                let g_prob: f64 = bad_vals
                    .iter()
                    .map(|&m| Self::pdf(val, m, sigma))
                    .sum::<f64>()
                    / bad_vals.len() as f64;

                // Avoid log(0)
                let l_prob = l_prob.max(1e-12);
                let g_prob = g_prob.max(1e-12);

                log_l += l_prob.ln();
                log_g += g_prob.ln();
            }

            // EI ~ l(x) / g(x) -> log EI ~ log l - log g
            let ei = log_l - log_g;
            candidates_vec.push(candidate.clone());
            if ei > best_ei || best_candidate.is_empty() {
                best_ei = ei;
                best_candidate = candidate;
            }
        }

        // Return best of N candidates
        StrategyAction::Evaluate(vec![best_candidate])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scotts_bandwidth_calculation() {
        // Test with known data
        // For n=10, stddev=1.0: σ = 1.06 × 1.0 × 10^(-0.2) = 1.06 × 0.631 ≈ 0.669
        let values: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let bandwidth = TPE::scotts_bandwidth(&values);

        // Values 0-9 have stddev ≈ 2.87
        // Expected: 1.06 × 2.87 × 10^(-0.2) ≈ 1.92
        assert!(
            bandwidth > 1.0 && bandwidth < 3.0,
            "Bandwidth should be reasonable, got {}",
            bandwidth
        );
    }

    #[test]
    fn test_scotts_bandwidth_adapts_to_distribution() {
        // Narrow distribution
        let narrow: Vec<f64> = (0..20).map(|i| 5.0 + (i as f64) * 0.01).collect();
        let narrow_bw = TPE::scotts_bandwidth(&narrow);

        // Wide distribution
        let wide: Vec<f64> = (0..20).map(|i| i as f64 * 10.0).collect();
        let wide_bw = TPE::scotts_bandwidth(&wide);

        assert!(
            narrow_bw < wide_bw,
            "Narrow distribution should have smaller bandwidth: {} vs {}",
            narrow_bw,
            wide_bw
        );
    }

    #[test]
    fn test_scotts_bandwidth_scales_with_n() {
        // Larger n should reduce bandwidth
        let small_n: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let large_n: Vec<f64> = (0..100).map(|i| (i % 10) as f64).collect(); // Same range

        let small_bw = TPE::scotts_bandwidth(&small_n);
        let large_bw = TPE::scotts_bandwidth(&large_n);

        assert!(
            large_bw < small_bw,
            "Larger n should reduce bandwidth: {} vs {}",
            large_bw,
            small_bw
        );
    }

    #[test]
    fn test_bandwidth_minimum_clamp() {
        // Very tight data should still have minimum bandwidth
        let tight: Vec<f64> = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let bw = TPE::scotts_bandwidth(&tight);

        assert!(bw >= 1e-6, "Bandwidth should be clamped to minimum");
    }

    #[test]
    fn test_silverman_bandwidth() {
        // Test Silverman's bandwidth rule
        let values: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let bandwidth = TPE::silverman_bandwidth(&values);

        // Should be reasonable
        assert!(
            bandwidth > 0.0 && bandwidth < 10.0,
            "Silverman bandwidth should be reasonable, got {}",
            bandwidth
        );
    }

    #[test]
    fn test_silverman_bandwidth_fallback() {
        // With < 4 samples, should fall back to Scott's
        let values: Vec<f64> = vec![1.0, 2.0, 3.0];
        let silverman_bw = TPE::silverman_bandwidth(&values);
        let scott_bw = TPE::scotts_bandwidth(&values);

        assert!((silverman_bw - scott_bw).abs() < 1e-10);
    }

    #[test]
    fn test_fixed_bandwidth() {
        let bw = TPE::fixed_bandwidth(10.0, 0.1);
        assert!((bw - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_fixed_bandwidth_minimum() {
        let bw = TPE::fixed_bandwidth(0.0, 0.1);
        assert!(bw >= 1e-6, "Fixed bandwidth should be clamped to minimum");
    }

    #[test]
    fn test_tpe_with_bandwidth_rule() {
        let tpe_scott = TPE::with_bandwidth_rule(2, BandwidthRule::Scott);
        assert_eq!(tpe_scott.bandwidth_rule, BandwidthRule::Scott);

        let tpe_silverman = TPE::with_bandwidth_rule(2, BandwidthRule::Silverman);
        assert_eq!(tpe_silverman.bandwidth_rule, BandwidthRule::Silverman);

        let tpe_fixed = TPE::with_bandwidth_rule(2, BandwidthRule::Fixed);
        assert_eq!(tpe_fixed.bandwidth_rule, BandwidthRule::Fixed);
    }

    #[test]
    fn test_scotts_bandwidth_edge_case() {
        // Test with < 2 values
        let one_val = vec![1.0];
        let bw = TPE::scotts_bandwidth(&one_val);
        assert_eq!(bw, 1.0, "Should return fallback 1.0 for insufficient data");

        let empty: Vec<f64> = vec![];
        let bw_empty = TPE::scotts_bandwidth(&empty);
        assert_eq!(bw_empty, 1.0, "Should return fallback 1.0 for empty data");
    }

    #[test]
    fn test_tpe_pdf_function() {
        // Test the Gaussian PDF calculation
        let pdf_at_mean = TPE::pdf(0.0, 0.0, 1.0);
        // At mean with sigma=1, PDF should be ~0.3989
        assert!(
            (pdf_at_mean - 0.3989).abs() < 0.01,
            "PDF at mean should be ~0.3989, got {}",
            pdf_at_mean
        );

        // PDF should decrease as we move away from mean
        let pdf_at_1std = TPE::pdf(1.0, 0.0, 1.0);
        assert!(pdf_at_1std < pdf_at_mean);
    }
}
