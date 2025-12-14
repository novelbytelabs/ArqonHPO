use crate::artifact::EvalTrace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Landscape {
    Structured,
    Chaotic,
}

pub trait Classify: Send + Sync {
    /// Classify the landscape based on probe history.
    /// Returns (Label, Score). Score > threshold implies Chaotic usually.
    fn classify(&self, history: &[EvalTrace]) -> (Landscape, f64);
}

// ============================================================================
// VarianceClassifier - Original MVP implementation (CV-based)
// ============================================================================

pub struct VarianceClassifier {
    pub threshold: f64,
}

impl Default for VarianceClassifier {
    fn default() -> Self {
        Self { threshold: 2.0 } // arbitrary default, tuned later
    }
}

impl Classify for VarianceClassifier {
    fn classify(&self, history: &[EvalTrace]) -> (Landscape, f64) {
        if history.is_empty() {
            return (Landscape::Chaotic, 1.0); // Default safe fallback
        }

        let values: Vec<f64> = history.iter().map(|t| t.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        // Coefficient of Variation (CV) = sigma / mu
        let cv = if mean.abs() > 1e-9 {
            variance.sqrt() / mean.abs()
        } else {
            variance.sqrt() // fallback if mean near zero
        };

        if cv < self.threshold {
            (Landscape::Structured, cv)
        } else {
            (Landscape::Chaotic, cv)
        }
    }
}

// ============================================================================
// ResidualDecayClassifier - RPZL algorithm (α decay analysis)
// ============================================================================

/// Classifies landscapes using residual decay analysis (RPZL methodology).
/// 
/// The algorithm measures how errors decrease across iterative refinement:
/// - For smooth/structured functions, errors decay geometrically (α < 0.5)
/// - For chaotic functions, errors do not decay consistently (α >= 0.5)
/// 
/// The α value is estimated by fitting an exponential decay curve to the
/// sequence of residuals between sorted objective values.
pub struct ResidualDecayClassifier {
    /// Threshold for α. Values below this are classified as Structured.
    /// Default: 0.5 (per spec clarification session 2025-12-14)
    pub alpha_threshold: f64,
    /// Minimum samples required for reliable estimation
    pub min_samples: usize,
}

impl Default for ResidualDecayClassifier {
    fn default() -> Self {
        Self {
            alpha_threshold: 0.5,
            min_samples: 5,
        }
    }
}

impl ResidualDecayClassifier {
    /// Create a new classifier with custom threshold
    pub fn with_threshold(alpha_threshold: f64) -> Self {
        Self {
            alpha_threshold,
            min_samples: 5,
        }
    }

    /// Estimate the decay rate α from a sequence of residuals.
    /// 
    /// Given residuals E_k, we fit E_k ≈ C × β^k where:
    /// - β is the decay factor (0 < β < 1 for decay)
    /// - α = -ln(β) is the decay rate
    /// 
    /// For geometric decay (smooth functions): α is small (< 0.5)
    /// For non-decay (chaotic functions): α is large or undefined
    fn estimate_alpha(&self, residuals: &[f64]) -> f64 {
        if residuals.len() < 2 {
            return 1.0; // Not enough data, assume chaotic
        }

        // Log-transform for linear regression: ln(E_k) = ln(C) + k*ln(β)
        // We estimate ln(β) as the slope, then α = -ln(β)
        let mut log_residuals: Vec<f64> = Vec::new();
        let mut indices: Vec<f64> = Vec::new();
        
        for (i, &r) in residuals.iter().enumerate() {
            if r > 1e-12 {
                log_residuals.push(r.ln());
                indices.push(i as f64);
            }
        }

        if log_residuals.len() < 2 {
            return 1.0; // All residuals near zero or negative
        }

        // Simple linear regression: y = a + b*x where b = ln(β)
        let n = log_residuals.len() as f64;
        let sum_x: f64 = indices.iter().sum();
        let sum_y: f64 = log_residuals.iter().sum();
        let sum_xy: f64 = indices.iter().zip(log_residuals.iter()).map(|(x, y)| x * y).sum();
        let sum_xx: f64 = indices.iter().map(|x| x * x).sum();

        let denom = n * sum_xx - sum_x * sum_x;
        if denom.abs() < 1e-12 {
            return 1.0; // Degenerate case
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom; // This is ln(β)
        
        // α = -ln(β) = -slope
        // For decay: slope < 0, so α > 0
        // For growth: slope > 0, so α < 0 (treat as chaotic)
        let alpha = -slope;

        // Clamp to reasonable range [0, 2]
        alpha.max(0.0).min(2.0)
    }

    /// Compute residuals from sorted objective values.
    /// 
    /// Residuals are the differences between consecutive sorted values,
    /// representing how much improvement occurs at each step.
    fn compute_residuals(&self, values: &[f64]) -> Vec<f64> {
        if values.len() < 2 {
            return vec![];
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Residuals: E_k = |sorted[k+1] - sorted[k]|
        sorted
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .collect()
    }
}

impl Classify for ResidualDecayClassifier {
    fn classify(&self, history: &[EvalTrace]) -> (Landscape, f64) {
        if history.len() < self.min_samples {
            // Not enough data for reliable estimation, default to chaotic (safer)
            return (Landscape::Chaotic, 1.0);
        }

        let values: Vec<f64> = history.iter().map(|t| t.value).collect();
        let residuals = self.compute_residuals(&values);
        
        if residuals.is_empty() {
            return (Landscape::Chaotic, 1.0);
        }

        let alpha = self.estimate_alpha(&residuals);

        // Classification per spec: α < 0.5 → Structured (geometric decay)
        if alpha < self.alpha_threshold {
            (Landscape::Structured, alpha)
        } else {
            (Landscape::Chaotic, alpha)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn trace(value: f64) -> EvalTrace {
        EvalTrace {
            phase: "test".to_string(),
            params: HashMap::new(),
            value,
            best_so_far: value,
        }
    }

    #[test]
    fn test_residual_decay_alpha_estimation() {
        let classifier = ResidualDecayClassifier::default();
        
        // Geometric decay: E_k = 10 * 0.5^k
        // This should have α = -ln(0.5) ≈ 0.693, but our residuals are
        // computed from sorted values, so we test with actual smooth function output
        let smooth_residuals = vec![1.0, 0.5, 0.25, 0.125, 0.0625];
        let alpha = classifier.estimate_alpha(&smooth_residuals);
        
        // Should be approximately ln(2) ≈ 0.693
        assert!(alpha > 0.5 && alpha < 1.0, "α for geometric decay should be ~0.69, got {}", alpha);
    }

    #[test]
    fn test_residual_decay_sphere_structured() {
        let classifier = ResidualDecayClassifier::default();
        
        // Sphere function samples: f(x) = x^2 for x = [-2, -1, 0, 1, 2]
        // Values: [4, 1, 0, 1, 4]
        // Sorted: [0, 1, 1, 4, 4]
        // Residuals: [1, 0, 3, 0] - not purely geometric
        // But with denser sampling, we get better decay
        let samples: Vec<EvalTrace> = (-10..=10)
            .map(|i| {
                let x = i as f64 * 0.4;
                trace(x * x)
            })
            .collect();
        
        let (landscape, alpha) = classifier.classify(&samples);
        
        // Sphere is smooth, should have good decay characteristics
        // With our algorithm, it should classify as Structured
        println!("Sphere α = {}", alpha);
        assert_eq!(landscape, Landscape::Structured, "Sphere should be Structured, α={}", alpha);
    }

    #[test]
    fn test_residual_decay_rastrigin_chaotic() {
        let classifier = ResidualDecayClassifier::default();
        
        // Rastrigin function: many local minima, high frequency oscillations
        use std::f64::consts::PI;
        let samples: Vec<EvalTrace> = (-10..=10)
            .map(|i| {
                let x = i as f64 * 0.4;
                let val = 10.0 + x * x - 10.0 * (2.0 * PI * x).cos();
                trace(val)
            })
            .collect();
        
        let (landscape, alpha) = classifier.classify(&samples);
        
        // Rastrigin has many local minima, residuals don't decay smoothly
        println!("Rastrigin α = {}", alpha);
        assert_eq!(landscape, Landscape::Chaotic, "Rastrigin should be Chaotic, α={}", alpha);
    }
}
