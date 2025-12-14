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
        
        // Simple heuristic: High variance relative to mean? Or just raw variance?
        // For MVP, raw variance might be unscaled.
        // Let's use Coefficient of Variation (CV) = sigma / mu, if mu != 0
        // Or just a placeholder score.
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
