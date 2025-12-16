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
    
    // 4. Run Simulation Loop
    let mut universe = Universe::new(12345);
    
    println!("Running {} steps...", STEPS);
    
    for step in 0..STEPS {
        let start_tick = Instant::now();
        
        // --- Tier 1 Executor Logic ---
        
        // A. Select Variant (Bandit)
        let context = Context::new()
            .with_latency_budget(1000)
            .with_load(0.5);
            
        let eligible = catalog.filter_eligible(
            context.latency_budget_us,
            u64::MAX,
            0.0,
            1.0,
            "cpu"
        );
        
        let selection = bandit.select(&eligible, catalog.default_variant().map(|v| v.id));
        let variant_id = selection.as_ref().map(|s| s.variant_id).unwrap_or(0); // Should handle unwrap safely in real impl
        
        if let Some(variant) = catalog.get(variant_id) {
            universe.apply_variant(variant);
        }
        
        // B. Apply Physics Propsals (Adaptive Engine)
        // Note: In a real loop, we'd apply the *previous* proposal here, then observe, then get *next* proposal.
        // Or get proposal -> apply -> observe -> tell.
        // AdaptiveEngine uses observe->proposal flow.
        
        // Getting current consolidated config
        let snapshot = adaptive_engine.current();
        universe.apply_physics(&snapshot.params);
        
        // C. Execute Workload
        let start_eval = Instant::now();
        let objective = universe.step();
        let eval_dur = start_eval.elapsed();
        
        // D. Feedback Loop
        // Telemetry Digest
        let digest = TelemetryDigest::objective(objective);
        
        // Update Bandit
        // Reward: normalized objective? If objective is in [0, 1], perfect.
        bandit.update(variant_id, objective);
        
        // Update Adaptive Engine
        let engine_start = Instant::now();
        if let Some(delta) = adaptive_engine.observe(digest) {
            // Check budget: if getting the proposal took too long, SKIP applying it?
            // Actually, `observe` returns the proposal. We typically apply it for the NEXT frame.
            match adaptive_engine.apply_delta(&delta) {
                Ok(_) => {
                    // Log success
                    serde_json::to_writer(&mut audit_log, &serde_json::json!({
                        "step": step,
                        "action": "update_params",
                        "delta": delta,
                        "status": "applied"
                    }))?;
                    writeln!(&mut audit_log)?;
                },
                Err(violation) => {
                    // Log violation
                    serde_json::to_writer(&mut audit_log, &serde_json::json!({
                        "step": step,
                        "action": "update_params",
                        "violation": format!("{:?}", violation),
                        "status": "rejected"
                    }))?;
                    writeln!(&mut audit_log)?;
                }
            }
        }
        let engine_dur = engine_start.elapsed();
        
        let total_tick_dur = start_tick.elapsed();
        
        // Reporting
        // csv: step, objective, diff_rate, noise, variant, engine_us, total_us
        csv_log.serialize((
            step,
            objective,
            universe.diffusion_rate,
            universe.noise_level,
            variant_id,
            engine_dur.as_micros(),
            total_tick_dur.as_micros()
        ))?;
        
        if step % 100 == 0 {
            println!("Step {}: Obj={:.4}, Engine={}us", step, objective, engine_dur.as_micros());
        }
    }
    
    println!("Simulation complete. Artifacts: audit_log.jsonl, history.csv");
    Ok(())
}
