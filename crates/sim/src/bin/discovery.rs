use arqonhpo_core::omega::{
    DiscoveryLoop, Evaluator, Candidate, DiscoverySource, EvaluationResult,
    MockLlmObserver, Observer, ObserverContext
};
use arqonhpo_core::variant_catalog::{Variant, VariantType, VariantConstraints};
use arqon_sim::Universe;
use std::collections::HashMap;

struct UniverseEvaluator;

impl Evaluator for UniverseEvaluator {
    fn evaluate(&self, candidate: &Candidate) -> EvaluationResult {
        // use seed for reproducibility
        let mut sim = Universe::new(12345);
        
        sim.apply_variant(&candidate.variant);
        
        // Run simulation for N steps
        let n_steps = 100;
        let start = std::time::Instant::now();
        let mut total_quality = 0.0;
        
        for _ in 0..n_steps {
            total_quality += sim.step(); // Returns normalized stability (0.0 to 1.0)
        }
        
        let avg_quality = total_quality / n_steps as f64;
        let duration_ms = start.elapsed().as_millis() as u64;
        
        // Define simple pass criteria
        // Recall we changed Universe::step to return (1.0 - mean_delta).
        // Stable systems have high score (~1.0).
        let passed_safety = avg_quality > 0.8; 
        
        EvaluationResult {
            candidate_id: candidate.id.clone(),
            score: avg_quality,
            metrics: HashMap::from([
                ("quality".to_string(), avg_quality),
                ("latency_ms".to_string(), duration_ms as f64),
            ]),
            passed_safety,
            duration_ms,
        }
    }
}

fn main() {
    println!("Starting Offline Discovery Loop (POC 4+5)...");
    
    let evaluator = UniverseEvaluator;
    let mut loop_runner = DiscoveryLoop::new(evaluator);
    
    // 1. POC 5: Consult Emergent Observer for a Candidate
    println!("Consulting Observer (Mock LLM)...");
    let observer = MockLlmObserver::new("gpt-4-simulation");
    
    let context = ObserverContext {
        recent_telemetry: vec![
            "Step 90: instability detected".to_string(),
            "Step 95: instability high".to_string(),
        ],
        current_config: r#"{ "kernel_radius": "1" }"#.to_string(),
        goal_description: "Propose a kernel that balances smoothing with edge retention.".to_string(),
    };
    
    if let Some(candidate) = observer.propose(&context) {
        println!("Observer proposed: {} (Source: {:?})", candidate.variant.name, candidate.source);
        println!("Hypothesis: {}", candidate.hypothesis);
        loop_runner.add_candidate(candidate);
    } else {
        println!("Observer remained silent.");
    }

    // 2. Add a baseline candidate manually for comparison
    let baseline_variant = Variant {
        id: 0,
        name: "baseline_kernel_3x3".to_string(),
        version: "1.0".to_string(),
        variant_type: VariantType::Kernel,
        constraints: VariantConstraints::default(),
        expected_latency_us: 150,
        is_default: false,
        metadata: HashMap::from([("kernel_radius".to_string(), "1".to_string())]),
    };
    
    loop_runner.add_candidate(Candidate {
        id: "cand_baseline".to_string(),
        source: DiscoverySource::Manual,
        variant: baseline_variant,
        hypothesis: "Baseline check".to_string(),
    });
    
    // 3. Run Loop
    println!("Evaluating candidates...");
    let promotions = loop_runner.step();
    
    // 4. Report
    println!("Discovery Step Complete.");
    println!("Promoted {} variants:", promotions.len());
    for p in promotions {
        println!(" - {} (Score: {})", p.name, p.metadata.get("omega_score").unwrap_or(&"?".to_string()));
    }
}
