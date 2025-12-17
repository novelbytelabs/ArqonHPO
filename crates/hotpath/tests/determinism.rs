use hotpath::spsa::{Spsa, SpsaConfig};

#[test]
fn test_spsa_determinism() {
    // Constitution VIII.3 Check: Deterministic Replay
    // "Given same seed and same registry => exact same sequence of deltas."

    let seed = 12345;
    let dim = 10;
    
    // Config with small window for easy testing
    let config = SpsaConfig {
        eval_window_digests: 2,
        ..SpsaConfig::default()
    };

    // Run A
    let mut spsa_a = Spsa::new(seed, dim, 0.1, 0.1, config.clone());
    
    // Run B
    let mut spsa_b = Spsa::new(seed, dim, 0.1, 0.1, config.clone());
    
    // --- Step 1: Generate Perturbation ---
    let delta_a1 = spsa_a.generate_perturbation();
    let delta_b1 = spsa_b.generate_perturbation();
    
    assert_eq!(delta_a1.as_slice(), delta_b1.as_slice(), "Run A and B diverged at perturbation generation");
    
    // Verify they are ready for + phase
    spsa_a.start_plus_perturbation(delta_a1.clone());
    spsa_b.start_plus_perturbation(delta_b1.clone());

    // --- Step 2: Telemetry (Plus Phase) ---
    // Feed identical telemetry
    spsa_a.record_objective(0.5);
    spsa_a.record_objective(0.6); // 2 samples > window of 1? No, window is 2.
    
    spsa_b.record_objective(0.5);
    spsa_b.record_objective(0.6);
    
    // Complete plus window
    let res_a = spsa_a.complete_eval_window();
    let res_b = spsa_b.complete_eval_window();
    
    assert!(res_a.is_none());
    assert!(res_b.is_none());

    // --- Step 3: Telemetry (Minus Phase) ---
    // Feed identical telemetry
    spsa_a.record_objective(0.4);
    spsa_a.record_objective(0.5);
    
    spsa_b.record_objective(0.4);
    spsa_b.record_objective(0.5);

    // Complete minus window -> Should produce update
    let update_a = spsa_a.complete_eval_window();
    let update_b = spsa_b.complete_eval_window();

    assert!(update_a.is_some());
    assert!(update_b.is_some());

    let (grad_a, delta_a_update) = update_a.unwrap();
    let (grad_b, delta_b_update) = update_b.unwrap();

    assert_eq!(grad_a.as_slice(), grad_b.as_slice(), "Gradients diverged");
    assert_eq!(delta_a_update.as_slice(), delta_b_update.as_slice(), "Update deltas diverged");
    
    // Verify state reset
    assert_eq!(spsa_a.iteration(), spsa_b.iteration());
    assert_eq!(spsa_a.iteration(), 1);
}
