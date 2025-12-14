//! Tests for classifier algorithms.
//!
//! Tests cover:
//! - Residual decay α estimation
//! - Sphere function → Structured classification
//! - Rastrigin function → Chaotic classification
//! - Determinism given same probe samples

use crate::artifact::EvalTrace;
use crate::classify::{Classify, Landscape, ResidualDecayClassifier, VarianceClassifier};
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
    (-10..=10)
        .map(|i| {
            let x = i as f64 * 0.4;
            trace(x * x) // Simple 1D sphere
        })
        .collect()
}

/// Helper to create probe samples for Rastrigin function (chaotic)
fn rastrigin_samples() -> Vec<EvalTrace> {
    // Rastrigin: many local minima, high frequency oscillations
    use std::f64::consts::PI;
    (-10..=10)
        .map(|i| {
            let x = i as f64 * 0.4;
            let val = 10.0 + x * x - 10.0 * (2.0 * PI * x).cos();
            trace(val)
        })
        .collect()
}

// ============================================================================
// VarianceClassifier Tests
// ============================================================================

#[test]
fn test_variance_classifier_structured_low_cv() {
    let classifier = VarianceClassifier { threshold: 2.0 };
    let samples: Vec<EvalTrace> = (0..10).map(|i| trace(10.0 + (i as f64) * 0.01)).collect();
    
    let (landscape, cv) = classifier.classify(&samples);
    
    assert!(cv < 2.0, "CV should be low for structured data");
    assert_eq!(landscape, Landscape::Structured);
}

#[test]
fn test_variance_classifier_chaotic_high_cv() {
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
    let classifier = VarianceClassifier::default();
    let samples = sphere_samples();
    
    let (landscape1, cv1) = classifier.classify(&samples);
    let (landscape2, cv2) = classifier.classify(&samples);
    
    assert_eq!(landscape1, landscape2);
    assert!((cv1 - cv2).abs() < 1e-10);
}

// ============================================================================
// ResidualDecayClassifier Tests (RPZL Algorithm)
// ============================================================================

#[test]
fn test_residual_decay_sphere_structured() {
    // Sphere function is smooth - should classify as Structured
    let classifier = ResidualDecayClassifier::default();
    let samples = sphere_samples();
    
    let (landscape, alpha) = classifier.classify(&samples);
    
    println!("Sphere α = {}", alpha);
    assert_eq!(landscape, Landscape::Structured, 
               "Sphere should be Structured, got α={}", alpha);
}

#[test]
fn test_residual_decay_rastrigin_chaotic() {
    // Rastrigin has many local minima - should classify as Chaotic
    let classifier = ResidualDecayClassifier::default();
    let samples = rastrigin_samples();
    
    let (landscape, alpha) = classifier.classify(&samples);
    
    println!("Rastrigin α = {}", alpha);
    assert_eq!(landscape, Landscape::Chaotic, 
               "Rastrigin should be Chaotic, got α={}", alpha);
}

#[test]
fn test_residual_decay_deterministic() {
    let classifier = ResidualDecayClassifier::default();
    let samples = sphere_samples();
    
    let (l1, a1) = classifier.classify(&samples);
    let (l2, a2) = classifier.classify(&samples);
    
    assert_eq!(l1, l2);
    assert!((a1 - a2).abs() < 1e-10, "Alpha should be identical for same input");
}

#[test]
fn test_residual_decay_insufficient_samples() {
    // With fewer than min_samples, should default to Chaotic
    let classifier = ResidualDecayClassifier::default();
    let samples = vec![trace(1.0), trace(2.0)]; // Only 2 samples
    
    let (landscape, _) = classifier.classify(&samples);
    
    assert_eq!(landscape, Landscape::Chaotic, 
               "Insufficient samples should default to Chaotic");
}
