//! Tests for TPE strategy.
//!
//! Tests cover:
//! - Scott's Rule bandwidth calculation
//! - Bandwidth adaptation across dimensions
//! - Deterministic sampling given seed

use crate::artifact::EvalTrace;
use crate::config::{SolverConfig, Domain, Scale};
use crate::strategies::{Strategy, StrategyAction};
use crate::strategies::tpe::{TPE, BandwidthRule};
use std::collections::HashMap;

/// Helper to create EvalTrace
fn trace(value: f64, x: f64) -> EvalTrace {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut params = HashMap::new();
    params.insert("x".to_string(), x);
    EvalTrace {
        eval_id: COUNTER.fetch_add(1, Ordering::SeqCst),
        params,
        value,
        cost: 1.0,
    }
}

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
        budget: 100,
        seed: 42,
        probe_ratio: 0.2,
        strategy_params: None,
    }
}

#[test]
fn test_tpe_returns_evaluate_action() {
    let mut tpe = TPE::new(1);
    let config = test_config();
    
    // Create enough history for TPE to build model
    let history: Vec<EvalTrace> = (0..30)
        .map(|i| {
            let x = -5.0 + (i as f64) * 0.33;
            trace(x * x, x)
        })
        .collect();
    
    let action = tpe.step(&config, &history);
    
    match action {
        StrategyAction::Evaluate(candidates) => {
            assert!(!candidates.is_empty(), "Should return at least one candidate");
            let x = candidates[0].get("x").expect("Should have x parameter");
            assert!(*x >= -5.0 && *x <= 5.0, "Candidate should be in bounds");
        }
        _ => panic!("Expected Evaluate action"),
    }
}

#[test]
fn test_tpe_deterministic() {
    let config = test_config();
    let history: Vec<EvalTrace> = (0..30)
        .map(|i| {
            let x = -5.0 + (i as f64) * 0.33;
            trace(x * x, x)
        })
        .collect();
    
    let mut tpe1 = TPE::new(1);
    let mut tpe2 = TPE::new(1);
    
    let action1 = tpe1.step(&config, &history);
    let action2 = tpe2.step(&config, &history);
    
    match (action1, action2) {
        (StrategyAction::Evaluate(c1), StrategyAction::Evaluate(c2)) => {
            let x1 = c1[0].get("x").unwrap();
            let x2 = c2[0].get("x").unwrap();
            assert!((x1 - x2).abs() < 1e-10, "Same seed should produce same candidate");
        }
        _ => panic!("Expected Evaluate actions"),
    }
}

// ============================================================================
// Scott's Rule Tests (now implemented!)
// ============================================================================

#[test]
fn test_scotts_rule_bandwidth_calculation() {
    // Test that σ = 1.06 × stddev × n^(-1/5)
    let values: Vec<f64> = (0..10).map(|i| i as f64).collect();
    let bandwidth = TPE::scotts_bandwidth(&values);
    
    // Values 0-9 have stddev ≈ 2.87
    // Expected: 1.06 × 2.87 × 10^(-0.2) ≈ 1.92
    assert!(bandwidth > 1.0 && bandwidth < 3.0, 
            "Bandwidth should be reasonable, got {}", bandwidth);
}

#[test]
fn test_scotts_rule_adapts_to_distribution() {
    // Narrow distribution should have smaller bandwidth
    let narrow: Vec<f64> = (0..20).map(|i| 5.0 + (i as f64) * 0.01).collect();
    let narrow_bw = TPE::scotts_bandwidth(&narrow);

    // Wide distribution should have larger bandwidth
    let wide: Vec<f64> = (0..20).map(|i| i as f64 * 10.0).collect();
    let wide_bw = TPE::scotts_bandwidth(&wide);

    assert!(narrow_bw < wide_bw, 
            "Narrow distribution should have smaller bandwidth");
}

#[test]
fn test_tpe_with_bandwidth_rule() {
    // Test that different bandwidth rules can be used
    let tpe_scott = TPE::with_bandwidth_rule(1, BandwidthRule::Scott);
    let tpe_fixed = TPE::with_bandwidth_rule(1, BandwidthRule::Fixed);
    
    // Both should create valid TPE instances
    assert_eq!(tpe_scott.bandwidth_rule, BandwidthRule::Scott);
    assert_eq!(tpe_fixed.bandwidth_rule, BandwidthRule::Fixed);
}

