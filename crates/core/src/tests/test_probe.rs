//! Tests for probe strategies.
//!
//! Tests cover:
//! - Prime sequence generation
//! - Deterministic sampling
//! - Multi-scale coverage

use crate::config::{SolverConfig, Domain, Scale};
use crate::probe::{Probe, UniformProbe};
use std::collections::HashMap;

/// Create a basic config for testing
fn test_config() -> SolverConfig {
    let mut bounds = HashMap::new();
    bounds.insert("x".to_string(), Domain {
        min: -5.0,
        max: 5.0,
        scale: Scale::Linear,
    });
    
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
        assert!((x1 - x2).abs() < 1e-10, "Same seed should produce same samples");
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
    
    assert_eq!(samples.len(), expected, "Should generate correct number of samples");
}

// ============================================================================
// FAILING TESTS FOR PRIME-INDEX PROBE (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "PrimeIndexProbe not yet implemented - T031-T033"]
fn test_prime_sequence_generation() {
    // Should generate correct prime sequence: 2, 3, 5, 7, 11, 13, ...
    todo!("Implement Sieve of Eratosthenes");
}

#[test]
#[ignore = "PrimeIndexProbe not yet implemented - T031-T033"]
fn test_prime_index_probe_deterministic() {
    // Same seed should produce same prime-indexed samples
    todo!("Implement PrimeIndexProbe");
}

#[test]
#[ignore = "PrimeIndexProbe not yet implemented - T031-T033"]
fn test_prime_index_probe_multi_scale() {
    // Prime ratios should provide multi-scale coverage
    // Samples at 2/N, 3/N, 5/N, 7/N, ... should not alias
    todo!("Implement PrimeIndexProbe");
}

#[test]
#[ignore = "PrimeIndexProbe not yet implemented - T031-T033"]
fn test_prime_index_probe_respects_bounds() {
    // All samples should be within configured bounds
    todo!("Implement PrimeIndexProbe");
}
