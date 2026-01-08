//! Tests for probe strategies.
//!
//! Tests cover:
//! - Prime sequence generation
//! - Deterministic sampling
//! - Multi-scale coverage

use crate::config::{Domain, Scale, SolverConfig};
use crate::probe::{Probe, UniformProbe};
use std::collections::HashMap;

/// Create a basic config for testing
fn test_config() -> SolverConfig {
    let mut bounds = HashMap::new();
    bounds.insert(
        "x".to_string(),
        Domain {
            min: -5.0,
            max: 5.0,
            scale: Scale::Linear,
        },
    );

    SolverConfig {
        bounds,
        budget: 50,
        seed: 42,
        probe_ratio: 0.2,
        strategy_params: None,
    }
}

#[test]
fn test_uniform_probe_deterministic() {
    let config = test_config();
    let probe = UniformProbe;

    let samples1 = probe.sample(&config);
    let samples2 = probe.sample(&config);

    assert_eq!(samples1.len(), samples2.len());

    for (s1, s2) in samples1.iter().zip(samples2.iter()) {
        let x1 = s1.get("x").unwrap();
        let x2 = s2.get("x").unwrap();
        assert!(
            (x1 - x2).abs() < 1e-10,
            "Same seed should produce same samples"
        );
    }
}

#[test]
fn test_uniform_probe_respects_bounds() {
    let config = test_config();
    let probe = UniformProbe;

    let samples = probe.sample(&config);

    for sample in samples {
        let x = sample.get("x").unwrap();
        assert!(*x >= -5.0 && *x <= 5.0, "Sample should be within bounds");
    }
}

#[test]
fn test_uniform_probe_sample_count() {
    let config = test_config();
    let probe = UniformProbe;

    let samples = probe.sample(&config);
    let expected = (config.budget as f64 * config.probe_ratio).ceil() as usize;

    assert_eq!(
        samples.len(),
        expected,
        "Should generate correct number of samples"
    );
}

// ============================================================================
// PRIME-INDEX PROBE TESTS (Implementation complete)
// ============================================================================

#[test]
fn test_prime_sequence_generation() {
    // Should generate correct prime sequence: 2, 3, 5, 7, 11, 13, ...
    use crate::probe::PrimeIndexProbe;
    
    let primes = PrimeIndexProbe::sieve_of_eratosthenes(30);
    assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
}

#[test]
fn test_prime_index_probe_deterministic_integration() {
    // Same seed should produce same prime-indexed samples
    use crate::probe::PrimeIndexProbe;
    
    let config = test_config();
    let probe = PrimeIndexProbe::new();
    
    let samples1 = probe.sample(&config);
    let samples2 = probe.sample(&config);
    
    assert_eq!(samples1.len(), samples2.len());
    for (s1, s2) in samples1.iter().zip(samples2.iter()) {
        let x1 = s1.get("x").unwrap();
        let x2 = s2.get("x").unwrap();
        assert!((x1 - x2).abs() < 1e-10, "Same seed should produce same samples");
    }
}

#[test]
fn test_prime_index_probe_multi_scale() {
    // Prime ratios should provide multi-scale coverage
    use crate::probe::PrimeIndexProbe;
    
    let config = test_config();
    let probe = PrimeIndexProbe::new();
    
    let samples = probe.sample(&config);
    let values: Vec<f64> = samples.iter().map(|s| *s.get("x").unwrap()).collect();
    
    // Check samples cover multiple regions (at least 50% of range)
    let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let coverage = (max_val - min_val) / 10.0;
    assert!(coverage > 0.5, "Prime samples should cover at least 50% of range");
}

#[test]
fn test_prime_index_probe_respects_bounds_integration() {
    // All samples should be within configured bounds
    use crate::probe::PrimeIndexProbe;
    
    let config = test_config();
    let probe = PrimeIndexProbe::new();
    
    let samples = probe.sample(&config);
    for sample in samples {
        let x = sample.get("x").unwrap();
        assert!(*x >= -5.0 && *x <= 5.0, "Sample should be within bounds");
    }
}
