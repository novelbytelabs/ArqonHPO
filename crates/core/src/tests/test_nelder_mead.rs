//! Tests for Nelder-Mead strategy.
//!
//! Tests cover:
//! - All 5 NM operations (Reflection, Expansion, Contraction, Shrink)
//! - Simplex convergence on smooth functions
//! - State machine transitions
//! - Probe seeding

use crate::artifact::EvalTrace;
use crate::config::{Domain, Scale, SolverConfig};
use crate::strategies::nelder_mead::NelderMead;
use crate::strategies::{Strategy, StrategyAction};
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
    bounds.insert(
        "x".to_string(),
        Domain {
            min: -5.0,
            max: 5.0,
            scale: Scale::Linear,
        },
    );
    bounds.insert(
        "y".to_string(),
        Domain {
            min: -5.0,
            max: 5.0,
            scale: Scale::Linear,
        },
    );

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
        trace(1.0, 0.0, 0.0), // Best
        trace(2.0, 1.0, 0.0), // Second
        trace(5.0, 0.0, 1.0), // Worst
    ]
}

#[test]
fn test_nelder_mead_init_builds_simplex() {
    let dim = 2;
    let mut nm = NelderMead::new(dim, vec![false; dim]);
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

    let mut nm1 = NelderMead::new(2, vec![false; 2]);
    let mut nm2 = NelderMead::new(2, vec![false; 2]);

    let action1 = nm1.step(&config, &history);
    let action2 = nm2.step(&config, &history);

    match (action1, action2) {
        (StrategyAction::Evaluate(c1), StrategyAction::Evaluate(c2)) => {
            let x1 = c1[0].get("x").unwrap();
            let x2 = c2[0].get("x").unwrap();
            assert!(
                (x1 - x2).abs() < 1e-10,
                "Same input should produce same output"
            );
        }
        (StrategyAction::Wait, StrategyAction::Wait) => {}
        (StrategyAction::Converged, StrategyAction::Converged) => {}
        _ => panic!("Actions should match"),
    }
}

// ============================================================================
// NELDER-MEAD OPERATION TESTS (Implementation complete)
// ============================================================================

#[test]
fn test_nelder_mead_expansion() {
    // If reflection is better than best, should try expansion
    // x_e = centroid + γ*(reflection - centroid) where γ=2.0
    let dim = 2;
    let nm = NelderMead::new(dim, vec![false; dim]);
    let centroid = vec![1.0, 1.0];
    let reflection = vec![2.0, 2.0];
    
    let expansion = nm.compute_expansion(&centroid, &reflection);
    
    // e = [1,1] + 2.0*([2,2] - [1,1]) = [1,1] + [2,2] = [3, 3]
    assert_eq!(expansion, vec![3.0, 3.0]);
}

#[test]
fn test_nelder_mead_outside_contraction() {
    // If reflection between second-worst and worst, try outside contraction
    // x_c = centroid + ρ*(reflection - centroid) where ρ=0.5
    let dim = 2;
    let nm = NelderMead::new(dim, vec![false; dim]);
    let centroid = vec![1.0, 1.0];
    let reflection = vec![2.0, 2.0];
    
    let contraction = nm.compute_outside_contraction(&centroid, &reflection);
    
    // c_o = [1,1] + 0.5*([2,2] - [1,1]) = [1,1] + [0.5,0.5] = [1.5, 1.5]
    assert_eq!(contraction, vec![1.5, 1.5]);
}

#[test]
fn test_nelder_mead_inside_contraction() {
    // If reflection worse than worst, try inside contraction
    // x_c = centroid + ρ*(worst - centroid) where ρ=0.5
    let dim = 2;
    let nm = NelderMead::new(dim, vec![false; dim]);
    let centroid = vec![1.0, 1.0];
    let worst = vec![0.0, 0.0];
    
    let contraction = nm.compute_inside_contraction(&centroid, &worst);
    
    // c_i = [1,1] + 0.5*([0,0] - [1,1]) = [1,1] + [-0.5,-0.5] = [0.5, 0.5]
    assert_eq!(contraction, vec![0.5, 0.5]);
}

#[test]
fn test_nelder_mead_shrink_calculation() {
    // If contraction fails, shrink all points toward best
    // x_i = best + σ*(x_i - best) where σ=0.5
    let dim = 2;
    let mut nm = NelderMead::new(dim, vec![false; dim]);
    nm.simplex = vec![
        (1.0, vec![0.0, 0.0]), // Best
        (2.0, vec![2.0, 0.0]), // Second
        (3.0, vec![0.0, 2.0]), // Worst
    ];
    
    let shrunk = nm.compute_shrunk_points();
    
    // shrunk[0] = [0,0] + 0.5*([2,0] - [0,0]) = [1, 0]
    // shrunk[1] = [0,0] + 0.5*([0,2] - [0,0]) = [0, 1]
    assert_eq!(shrunk.len(), 2);
    assert_eq!(shrunk[0], vec![1.0, 0.0]);
    assert_eq!(shrunk[1], vec![0.0, 1.0]);
}

#[test]
fn test_nelder_mead_convergence_detection() {
    // Should detect convergence when simplex diameter < ε
    let dim = 2;
    let mut nm = NelderMead::new(dim, vec![false; dim]);
    nm.tolerance = 1e-6;
    
    // Simplex with diameter < tolerance
    nm.simplex = vec![
        (1.0, vec![0.5, 0.5]),
        (1.0, vec![0.5 + 1e-8, 0.5]),
        (1.0, vec![0.5, 0.5 + 1e-8]),
    ];
    
    assert!(nm.check_convergence(), "Should detect convergence for tight simplex");
}

#[test]
fn test_nelder_mead_no_convergence_when_spread() {
    // Should NOT converge when simplex is spread
    let dim = 2;
    let mut nm = NelderMead::new(dim, vec![false; dim]);
    nm.tolerance = 1e-6;
    
    // Simplex with large diameter
    nm.simplex = vec![
        (1.0, vec![0.0, 0.0]),
        (2.0, vec![1.0, 0.0]),
        (3.0, vec![0.0, 1.0]),
    ];
    
    assert!(!nm.check_convergence(), "Should not converge for spread simplex");
}

#[test]
fn test_nelder_mead_seeded_from_probe_points() {
    // NM should use seed points as initial simplex vertices
    let dim = 2;
    let seeds = vec![
        (1.0, vec![0.1, 0.1]),
        (2.0, vec![0.2, 0.2]),
        (3.0, vec![0.3, 0.3]),
    ];
    
    let nm = NelderMead::with_seed_points(dim, seeds.clone(), vec![false; dim]);
    
    assert_eq!(nm.simplex.len(), 3);
    assert_eq!(nm.simplex[0], seeds[0]);
    assert_eq!(nm.simplex[1], seeds[1]);
    assert_eq!(nm.simplex[2], seeds[2]);
}
