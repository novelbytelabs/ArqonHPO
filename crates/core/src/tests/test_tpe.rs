//! Tests for TPE strategy.
//!
//! Tests cover:
//! - Scott's Rule bandwidth calculation
//! - Bandwidth adaptation across dimensions
//! - Deterministic sampling given seed

use crate::artifact::EvalTrace;
use crate::config::{SolverConfig, Domain, Scale};
use crate::strategies::{Strategy, StrategyAction};
use crate::strategies::tpe::TPE;
use std::collections::HashMap;

/// Helper to create EvalTrace
fn trace(value: f64, x: f64) -> EvalTrace {
    let mut params = HashMap::new();
    params.insert("x".to_string(), x);
    EvalTrace {
        phase: "test".to_string(),
        params,
        value,
        best_so_far: value,
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
// FAILING TESTS FOR SCOTT'S RULE BANDWIDTH (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "Scott's Rule not yet implemented - T016-T018"]
fn test_scotts_rule_bandwidth_calculation() {
    // Test that σ = 1.06 × stddev × n^(-1/5)
    // This test will fail until scotts_bandwidth is implemented
    todo!("Implement scotts_bandwidth function");
}

#[test]
#[ignore = "Scott's Rule not yet implemented - T016-T018"]
fn test_scotts_rule_adapts_to_distribution() {
    // Narrow distribution should have smaller bandwidth
    // Wide distribution should have larger bandwidth
    todo!("Implement scotts_bandwidth function");
}

#[test]
#[ignore = "Scott's Rule not yet implemented - T016-T018"]
fn test_scotts_rule_per_dimension() {
    // Each dimension should have its own bandwidth based on that dimension's data
    todo!("Implement scotts_bandwidth function");
}
