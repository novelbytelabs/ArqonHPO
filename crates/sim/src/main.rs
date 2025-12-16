use arqonhpo_core::adaptive_engine::{
    AdaptiveEngine, AdaptiveEngineConfig, TelemetryDigest
};
use arqonhpo_core::variant_catalog::{
    VariantCatalog, Variant, VariantType, ContextualBandit, BanditConfig, Context
};
use arqonhpo_core::config::{Domain, Scale};
use arqonhpo_core::adaptive_engine::Guardrails;
use arqonhpo_core::adaptive_engine::AtomicConfig;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use arqon_sim::{Universe, ROWS, COLS};

const STEPS: usize = 1000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Arqon Verification Simulation...");
    
    // 1. Setup Audit Logging
    let mut audit_log = File::create("audit_log.jsonl")?;
    let mut csv_log = csv::Writer::from_path("history.csv")?;
    
    // 2. Setup POC 1: Adaptive Engine
    let mut bounds = HashMap::new();
    bounds.insert("diffusion_rate".to_string(), Domain { min: 0.0, max: 1.0, scale: Scale::Linear });
    bounds.insert("noise_level".to_string(), Domain { min: 0.0, max: 0.2, scale: Scale::Linear });
    
    let engine_config = AdaptiveEngineConfig {
        seed: 42,
        bounds: bounds.clone(),
        learning_rate: 0.01,
        perturbation_scale: 0.01,
        budget_us: 1000, // 1ms budget
        guardrails: Guardrails::default(),
    };
    
    let initial_params = HashMap::from([
        ("diffusion_rate".to_string(), 0.5),
        ("noise_level".to_string(), 0.1),
    ]);
    
    let mut adaptive_engine = AdaptiveEngine::new(engine_config, initial_params);
    
    // 3. Setup POC 2: Variant Catalog
    let mut catalog = VariantCatalog::new();
    
    let v1 = Variant {
        id: 0, 
        name: "kernel_3x3_basic".to_string(),
        version: "1.0".to_string(),
        variant_type: VariantType::Kernel,
        constraints: Default::default(),
        expected_latency_us: 100,
        is_default: true,
        metadata: HashMap::new(),
    };
    let v2 = Variant {
        id: 0,
        name: "kernel_5x5_wide".to_string(),
        version: "1.0".to_string(),
        variant_type: VariantType::Kernel,
        constraints: Default::default(),
        expected_latency_us: 300,
        is_default: false,
        metadata: HashMap::new(),
    };
    
    catalog.add(v1);
    catalog.add(v2);
    
    let mut bandit = ContextualBandit::new(BanditConfig::default());
    let recovery_threshold = 0.90;
    
    // --- Baseline Run (No Adaptation) ---
    println!("\n=== Running Baseline (No Adaptation) ===");
    let mut universe_base = Universe::new(12345);
    let mut baseline_recovery_steps = 0;
    let mut baseline_shocked = false;
    
    for step in 0..STEPS {
        if step == 500 {
            universe_base.inject_shock();
            baseline_shocked = true;
        }
        let q = universe_base.step();
        if baseline_shocked && q > recovery_threshold {
            if baseline_recovery_steps == 0 {
                baseline_recovery_steps = step - 500;
            }
        }
    }
    println!("Baseline Recovery: {} steps", baseline_recovery_steps);


    // --- Adaptive Run ---
    println!("\n=== Running Adaptive System ===");
    let mut universe = Universe::new(12345);
    
    // Detailed Latency trackers
    let mut t2_decision_us = Vec::with_capacity(STEPS);
    let mut t1_compute_us = Vec::with_capacity(STEPS); // In-memory
    let mut t1_audit_us = Vec::with_capacity(STEPS);   // Disk I/O
    let mut e2e_us = Vec::with_capacity(STEPS);
    let mut total_deltas = 0;
    let mut applied_updates = 0;
    
    // Recovery tracker
    let mut shocked = false;
    let mut recovery_start = 0;
    let mut recovery_steps = 0;
    let recovery_threshold = 0.90;

    println!("Running {} steps with Shock at 500...", STEPS);
    
    for step in 0..STEPS {
        // --- Shock Injection ---
        if step == 500 {
            println!("!!! INJECTING SHOCK !!!");
            universe.inject_shock();
            shocked = true;
            recovery_start = step;
        }

        let loop_start = Instant::now(); // Start of control tick visibility
        
        // --- Tier 1 Executor Logic (Apply Previous) ---
        // (In this synchronous sim, we apply 'current' from engine which was updated last step)
        let snapshot = adaptive_engine.current();
        universe.apply_physics(&snapshot.params); // Dataplane observes config
        
        let dataplane_observed_ts = Instant::now();
        
        // --- Workload ---
        let objective = universe.step();
        
        if shocked && objective > recovery_threshold {
            if recovery_steps == 0 {
                recovery_steps = step - recovery_start;
                println!("Recovered in {} steps (Quality > {})", recovery_steps, recovery_threshold);
            }
        }
        
        // --- Feedback ---
        let digest = TelemetryDigest::objective(objective);
        bandit.update(0, objective); // simplified variant tracking
        
        // --- Adaptive Engine (T2 + T1) ---
        
        // 1. T2 Decision
        let t2_start = Instant::now();
        let proposal_opt = adaptive_engine.observe(digest);
        let t2_end = Instant::now();
        t2_decision_us.push(t2_end.duration_since(t2_start).as_micros() as u64);

        if let Some(delta) = proposal_opt {
            // 2. T1 Apply (In-Memory)
            let t1_compute_start = Instant::now();
            let apply_result = adaptive_engine.apply_delta(&delta);
            let t1_compute_end = Instant::now();
            
            t1_compute_us.push(t1_compute_end.duration_since(t1_compute_start).as_micros() as u64);
            
            // 3. T1 Audit (Durability)
            let t1_audit_start = Instant::now();
            match apply_result {
                Ok(_) => {
                    applied_updates += 1;
                    total_deltas += 1; // Simplify delta mag tracking for now
                    serde_json::to_writer(&mut audit_log, &serde_json::json!({
                        "step": step,
                        "action": "update",
                        "status": "applied"
                    }))?;
                    writeln!(&mut audit_log)?;
                },
                Err(_) => {}
            }
            let t1_audit_end = Instant::now();
            t1_audit_us.push(t1_audit_end.duration_since(t1_audit_start).as_micros() as u64);
            
            // E2E: From 'digest available' (t2_start) to 'config applied' (t1_compute_end)
            // Note: In this loop, the *next* step sees the config.
            // But latency is T2 + T1_compute.
            let e2e = t1_compute_end.duration_since(t2_start);
            e2e_us.push(e2e.as_micros() as u64);
            
        } else {
             t1_compute_us.push(0);
             t1_audit_us.push(0);
             e2e_us.push(t2_end.duration_since(t2_start).as_micros() as u64);
        }
        
        // Reporting
        csv_log.serialize((
            step,
            objective,
            universe.diffusion_rate,
            universe.noise_level,
            0,
            t2_end.duration_since(t2_start).as_micros(),
            dataplane_observed_ts.duration_since(loop_start).as_micros()
        ))?;
    }
    
    // Stats
    let sort_get = |vec: &mut Vec<u64>, pct: usize| -> u64 {
        vec.sort();
        if vec.is_empty() { return 0; }
        vec[vec.len() * pct / 100]
    };
    
    let t2_p99 = sort_get(&mut t2_decision_us, 99);
    let t1_comp_p99 = sort_get(&mut t1_compute_us, 99);
    let t1_audit_p99 = sort_get(&mut t1_audit_us, 99);
    let e2e_p99 = sort_get(&mut e2e_us, 99);
    
    println!("\n--- FINAL METRICS ---");
    println!("Baseline Recovery: {} steps", baseline_recovery_steps);
    println!("Adaptive Recovery: {} steps", recovery_steps);
    println!("\nLatency (p99):");
    println!("  T2 Decision: {} us", t2_p99);
    println!("  T1 Apply (Mem): {} us", t1_comp_p99);
    println!("  T1 Audit (Disk): {} us", t1_audit_p99);
    println!("  E2E Visible: {} us", e2e_p99);
    
    println!("\nActuation:");
    println!("  Updates Applied: {}", applied_updates);
    println!("  Variants Switched: (Bandit fixed to single for stress)");
    
    println!("Simulation complete.");
    Ok(())
}
