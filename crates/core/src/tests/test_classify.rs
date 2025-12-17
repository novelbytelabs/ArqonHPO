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
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    EvalTrace {
        eval_id: COUNTER.fetch_add(1, Ordering::SeqCst),
        params: HashMap::new(),
        value,
        cost: 1.0,
    }
}

/// Helper to create probe samples for Sphere function (smooth, structured)
/// Uses geometric spacing: 0.001, 0.002, 0.004, 0.008, ...
/// When sorted worst→best and residuals computed, this shows geometric DECAY
fn sphere_samples() -> Vec<EvalTrace> {
    // Geometric ratio 2: each value is double the previous
    // This represents a well-structured, smooth optimization landscape
    vec![
        trace(0.001),
        trace(0.002),
        trace(0.004),
        trace(0.008),
        trace(0.016),
        trace(0.032),
        trace(0.064),
        trace(0.128),
        trace(0.256),
        trace(0.512),
    ]
}

/// Helper to create probe samples for Rastrigin function (chaotic)
/// Uses linear spacing: 0, 1, 2, 3, 4... (constant residuals)
fn rastrigin_samples() -> Vec<EvalTrace> {
    // Linear spacing: residuals are all ~1 (flat, no decay)
    // This represents a chaotic landscape with many local optima
    vec![
        trace(0.0),
        trace(1.0),
        trace(2.0),
        trace(3.0),
        trace(4.0),
        trace(5.0),
        trace(6.0),
        trace(7.0),
        trace(8.0),
        trace(9.0),
    ]
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
    // High coefficient of variation should be Chaotic
    // CV = stddev / mean. For extreme values we get CV > 1.0
    // Using a threshold of 1.0 as a reasonable cutoff
    let classifier = VarianceClassifier { threshold: 1.0 };
    let samples: Vec<EvalTrace> = vec![
        trace(0.01),
        trace(1000.0),
        trace(0.02),
        trace(500.0),
        trace(0.01),
    ];

    let (landscape, cv) = classifier.classify(&samples);

    println!("Chaotic CV = {}", cv);
    assert!(cv >= 1.0, "CV should be high for chaotic data, got {}", cv);
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
// ResidualDecayClassifier Tests (PCR Algorithm)
// ============================================================================

#[test]
fn test_residual_decay_sphere_structured() {
    // Sphere function is smooth - should classify as Structured
    let classifier = ResidualDecayClassifier::default();
    let samples = sphere_samples();

    let (landscape, alpha) = classifier.classify(&samples);

    println!("Sphere α = {}", alpha);
    assert_eq!(
        landscape,
        Landscape::Structured,
        "Sphere should be Structured, got α={}",
        alpha
    );
}

#[test]
fn test_residual_decay_rastrigin_chaotic() {
    // Rastrigin has many local minima - should classify as Chaotic
    let classifier = ResidualDecayClassifier::default();
    let samples = rastrigin_samples();

    let (landscape, alpha) = classifier.classify(&samples);

    println!("Rastrigin α = {}", alpha);
    assert_eq!(
        landscape,
        Landscape::Chaotic,
        "Rastrigin should be Chaotic, got α={}",
        alpha
    );
}

#[test]
fn test_residual_decay_deterministic() {
    let classifier = ResidualDecayClassifier::default();
    let samples = sphere_samples();

    let (l1, a1) = classifier.classify(&samples);
    let (l2, a2) = classifier.classify(&samples);

    assert_eq!(l1, l2);
    assert!(
        (a1 - a2).abs() < 1e-10,
        "Alpha should be identical for same input"
    );
}

#[test]
fn test_residual_decay_insufficient_samples() {
    // With fewer than min_samples, should default to Chaotic
    let classifier = ResidualDecayClassifier::default();
    let samples = vec![trace(1.0), trace(2.0)]; // Only 2 samples

    let (landscape, _) = classifier.classify(&samples);

    assert_eq!(
        landscape,
        Landscape::Chaotic,
        "Insufficient samples should default to Chaotic"
    );
}
