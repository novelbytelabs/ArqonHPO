//! Tests for classifier algorithms.
//!
//! Tests cover:
//! - Residual decay α estimation
//! - Sphere function → Structured classification
//! - Rastrigin function → Chaotic classification
//! - Determinism given same probe samples

use crate::artifact::EvalTrace;
use crate::classify::{Classify, Landscape, VarianceClassifier};
use std::collections::HashMap;

/// Helper to create EvalTrace from value
fn trace(value: f64) -> EvalTrace {
    EvalTrace {
        phase: "test".to_string(),
        params: HashMap::new(),
        value,
        best_so_far: value,
    }
}

/// Helper to create probe samples for Sphere function (smooth, structured)
fn sphere_samples() -> Vec<EvalTrace> {
    // Sphere: f(x) = sum(x_i^2), very smooth
    // Sample at evenly spaced points
    (0..20)
        .map(|i| {
            let x = -5.0 + (i as f64) * 0.5;
            trace(x * x) // Simple 1D sphere
        })
        .collect()
}

/// Helper to create probe samples for Rastrigin function (chaotic)
fn rastrigin_samples() -> Vec<EvalTrace> {
    // Rastrigin: many local minima, high frequency oscillations
    use std::f64::consts::PI;
    (0..20)
        .map(|i| {
            let x = -5.0 + (i as f64) * 0.5;
            let val = 10.0 + x * x - 10.0 * (2.0 * PI * x).cos();
            trace(val)
        })
        .collect()
}

#[test]
fn test_variance_classifier_structured_low_cv() {
    // Low coefficient of variation should be Structured
    let classifier = VarianceClassifier { threshold: 2.0 };
    let samples: Vec<EvalTrace> = (0..10).map(|i| trace(10.0 + (i as f64) * 0.01)).collect();
    
    let (landscape, cv) = classifier.classify(&samples);
    
    assert!(cv < 2.0, "CV should be low for structured data");
    assert_eq!(landscape, Landscape::Structured);
}

#[test]
fn test_variance_classifier_chaotic_high_cv() {
    // High coefficient of variation should be Chaotic
    let classifier = VarianceClassifier { threshold: 2.0 };
    let samples: Vec<EvalTrace> = vec![
        trace(0.1), trace(100.0), trace(0.5), trace(50.0), trace(0.2)
    ];
    
    let (landscape, cv) = classifier.classify(&samples);
    
    assert!(cv >= 2.0, "CV should be high for chaotic data");
    assert_eq!(landscape, Landscape::Chaotic);
}

#[test]
fn test_classifier_deterministic() {
    // Same input should produce same output
    let classifier = VarianceClassifier::default();
    let samples = sphere_samples();
    
    let (landscape1, cv1) = classifier.classify(&samples);
    let (landscape2, cv2) = classifier.classify(&samples);
    
    assert_eq!(landscape1, landscape2);
    assert!((cv1 - cv2).abs() < 1e-10);
}

// ============================================================================
// FAILING TESTS FOR RESIDUAL DECAY CLASSIFIER (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "ResidualDecayClassifier not yet implemented - T010-T012"]
fn test_residual_decay_alpha_estimation() {
    // Test that α is estimated correctly from decay curve
    // For geometric decay E_k = C * β^k, α = -ln(β)
    // This test will fail until ResidualDecayClassifier is implemented
    todo!("Implement ResidualDecayClassifier");
}

#[test]
#[ignore = "ResidualDecayClassifier not yet implemented - T010-T012"]
fn test_residual_decay_sphere_structured() {
    // Sphere function should have geometric decay (α < 0.5)
    // This test will fail until ResidualDecayClassifier is implemented
    todo!("Implement ResidualDecayClassifier");
}

#[test]
#[ignore = "ResidualDecayClassifier not yet implemented - T010-T012"]
fn test_residual_decay_rastrigin_chaotic() {
    // Rastrigin function should NOT have geometric decay (α >= 0.5)
    // This test will fail until ResidualDecayClassifier is implemented
    todo!("Implement ResidualDecayClassifier");
}
