use arqonhpo_core::omega::{DiscoveryLoop, Evaluator, Candidate, DiscoverySource, EvaluationResult};
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
    println!("Starting Offline Discovery Loop (POC 4)...");
    
    let evaluator = UniverseEvaluator;
    let mut loop_runner = DiscoveryLoop::new(evaluator);
    
    // 1. Generate Candidates
    
    // Candidate A: Small kernel (stable)
    let good_variant = Variant {
        id: 0,
        name: "kernel_3x3_stable".to_string(),
        version: "2.0".to_string(),
        variant_type: VariantType::Kernel,
        constraints: VariantConstraints::default(),
        expected_latency_us: 150,
        is_default: false,
        metadata: HashMap::from([("kernel_radius".to_string(), "1".to_string())]),
    };
    
    loop_runner.add_candidate(Candidate {
        id: "cand_stable".to_string(),
        source: DiscoverySource::Heuristic,
        variant: good_variant,
        hypothesis: "Small kernel (radius 1) improves local stability".to_string(),
    });
    
    // Candidate B: Large kernel (unstable/chaotic?)
    // In our simplified physics large kernel might actually be MORE stable?
    // Let's test it. If it passes, it passes.
    // We add a 'noise_level' override in metadata to force instability for testing 'bad' candidate.
    // But Universe doesn't read noise_level from Variant metadata yet, only 'kernel_radius'.
    // See Universe::apply_variant(..).
    // I won't change Universe logic now, let's just see what happens.
    
    let big_variant = Variant {
        id: 0,
        name: "kernel_11x11_huge".to_string(),
        version: "0.1".to_string(),
        variant_type: VariantType::Kernel,
        constraints: VariantConstraints::default(),
        expected_latency_us: 2000,
        is_default: false,
        metadata: HashMap::from([("kernel_radius".to_string(), "5".to_string())]),
    };
    
     loop_runner.add_candidate(Candidate {
        id: "cand_huge".to_string(),
        source: DiscoverySource::LlmObserver,
        variant: big_variant,
        hypothesis: "Large kernel captures long-range interactions".to_string(),
    });
    
    // 2. Run Loop
    println!("Evaluating candidates...");
    let promotions = loop_runner.step();
    
    // 3. Report
    println!("Discovery Step Complete.");
    println!("Promoted {} variants:", promotions.len());
    for p in promotions {
        println!(" - {} (Score: {})", p.name, p.metadata.get("omega_score").unwrap_or(&"?".to_string()));
    }
}
