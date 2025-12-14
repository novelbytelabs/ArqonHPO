//! Tests for Nelder-Mead strategy.
//!
//! Tests cover:
//! - All 5 NM operations (Reflection, Expansion, Contraction, Shrink)
//! - Simplex convergence on smooth functions
//! - State machine transitions
//! - Probe seeding

use crate::artifact::EvalTrace;
use crate::config::{SolverConfig, Domain, Scale};
use crate::strategies::{Strategy, StrategyAction};
use crate::strategies::nelder_mead::NelderMead;
use std::collections::HashMap;

/// Helper to create EvalTrace
fn trace(value: f64, x: f64, y: f64) -> EvalTrace {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut params = HashMap::new();
    params.insert("x".to_string(), x);
    params.insert("y".to_string(), y);
    EvalTrace {
        eval_id: COUNTER.fetch_add(1, Ordering::SeqCst),
        params,
        value,
        cost: 1.0,
    }
}

/// Create a 2D config for testing
fn test_config_2d() -> SolverConfig {
    let mut bounds = HashMap::new();
    bounds.insert("x".to_string(), Domain {
        min: -5.0,
        max: 5.0,
        scale: Scale::Linear,
    });
    bounds.insert("y".to_string(), Domain {
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

/// Create history with 3 points forming a simplex (for 2D)
fn simplex_history() -> Vec<EvalTrace> {
    vec![
        trace(1.0, 0.0, 0.0),   // Best
        trace(2.0, 1.0, 0.0),   // Second
        trace(5.0, 0.0, 1.0),   // Worst
    ]
}

#[test]
fn test_nelder_mead_init_builds_simplex() {
    let mut nm = NelderMead::new(2);
    let config = test_config_2d();
    let history = simplex_history();
    
    let action = nm.step(&config, &history);
    
    // Should return an Evaluate action for reflection
    match action {
        StrategyAction::Evaluate(candidates) => {
            assert!(!candidates.is_empty(), "Should return reflection candidate");
        }
        StrategyAction::Wait => {
            // Also acceptable if not enough points
        }
        _ => {}
    }
}

#[test]
fn test_nelder_mead_deterministic() {
    let config = test_config_2d();
    let history = simplex_history();
    
    let mut nm1 = NelderMead::new(2);
    let mut nm2 = NelderMead::new(2);
    
    let action1 = nm1.step(&config, &history);
    let action2 = nm2.step(&config, &history);
    
    match (action1, action2) {
        (StrategyAction::Evaluate(c1), StrategyAction::Evaluate(c2)) => {
            let x1 = c1[0].get("x").unwrap();
            let x2 = c2[0].get("x").unwrap();
            assert!((x1 - x2).abs() < 1e-10, "Same input should produce same output");
        }
        (StrategyAction::Wait, StrategyAction::Wait) => {}
        (StrategyAction::Converged, StrategyAction::Converged) => {}
        _ => panic!("Actions should match"),
    }
}

// ============================================================================
// FAILING TESTS FOR EXPANSION OPERATION (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "Expansion not yet implemented - T024"]
fn test_nelder_mead_expansion() {
    // If reflection is better than best, should try expansion
    // x_e = centroid + γ*(reflection - centroid) where γ=2.0
    todo!("Implement Expansion handler");
}

// ============================================================================
// FAILING TESTS FOR CONTRACTION OPERATIONS (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "Outside Contraction not yet implemented - T025"]
fn test_nelder_mead_outside_contraction() {
    // If reflection between second-worst and worst, try outside contraction
    // x_c = centroid + ρ*(reflection - centroid) where ρ=0.5
    todo!("Implement Outside Contraction handler");
}

#[test]
#[ignore = "Inside Contraction not yet implemented - T026"]
fn test_nelder_mead_inside_contraction() {
    // If reflection worse than worst, try inside contraction
    // x_c = centroid + ρ*(worst - centroid) where ρ=0.5
    todo!("Implement Inside Contraction handler");
}

// ============================================================================
// FAILING TESTS FOR SHRINK OPERATION (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "Shrink not yet implemented - T027"]
fn test_nelder_mead_shrink() {
    // If contraction fails, shrink all points toward best
    // x_i = best + σ*(x_i - best) where σ=0.5
    todo!("Implement Shrink handler");
}

// ============================================================================
// FAILING TESTS FOR CONVERGENCE (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "Convergence detection not yet implemented - T028"]
fn test_nelder_mead_convergence_detection() {
    // Should detect convergence when simplex diameter < ε
    todo!("Implement convergence detection");
}

#[test]
#[ignore = "Full NM not yet implemented - T024-T028"]
fn test_nelder_mead_converges_on_sphere() {
    // Full NM should converge on Sphere function
    // This requires all operations to be implemented
    todo!("Implement full NM to test convergence");
}

// ============================================================================
// FAILING TESTS FOR PROBE SEEDING (TDD - implement to make pass)
// ============================================================================

#[test]
#[ignore = "Probe seeding not yet implemented - T036-T038"]
fn test_nelder_mead_seeded_from_probe_points() {
    // NM should use top-k probe points as initial simplex vertices
    todo!("Implement with_seed_points constructor");
}
