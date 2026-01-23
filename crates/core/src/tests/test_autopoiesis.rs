//! Comprehensive tests for Autopoiesis modules (Tier Ω, Tier 2, Governance)
//!
//! Test coverage targets:
//! - Unit tests for each function
//! - Integration tests for cross-module flows
//! - End-to-end test for complete pipeline

use crate::governance::{
    AlertSeverity, DownstreamMessage, EnforcementAction, RolloutConfig, UpstreamMessage,
};
use crate::omega::{
    Candidate, DiscoveryLoop, DiscoverySource, EvaluationResult, Evaluator, MockLlmObserver,
    Observer, ObserverContext,
};
use crate::variant_catalog::{
    BanditConfig, ContextualBandit, SelectionReason, Variant, VariantCatalog, VariantConstraints,
    VariantType,
};
use std::collections::HashMap;

// =============================================================================
// UNIT TESTS: Omega Tier
// =============================================================================

mod omega_unit_tests {
    use super::*;

    /// Mock evaluator that passes candidates with score > threshold
    struct MockEvaluator {
        threshold: f64,
        base_score: f64,
    }

    impl MockEvaluator {
        fn new(threshold: f64, base_score: f64) -> Self {
            Self {
                threshold,
                base_score,
            }
        }
    }

    impl Evaluator for MockEvaluator {
        fn evaluate(&self, candidate: &Candidate) -> EvaluationResult {
            EvaluationResult {
                candidate_id: candidate.id.clone(),
                score: self.base_score,
                metrics: HashMap::new(),
                passed_safety: self.base_score > self.threshold,
                duration_ms: 100,
            }
        }
    }

    #[test]
    fn test_discovery_loop_creation() {
        let evaluator = MockEvaluator::new(0.5, 0.9);
        let loop_instance = DiscoveryLoop::new(evaluator);
        // Should be created successfully (no public way to inspect, but no panic)
        assert!(true);
    }

    #[test]
    fn test_candidate_creation() {
        let variant = Variant {
            id: 0,
            name: "test_variant".to_string(),
            version: "1.0".to_string(),
            variant_type: VariantType::Kernel,
            constraints: VariantConstraints::default(),
            expected_latency_us: 1000,
            is_default: false,
            metadata: HashMap::new(),
        };

        let candidate = Candidate {
            id: "cand_001".to_string(),
            source: DiscoverySource::Heuristic,
            variant,
            hypothesis: "Test hypothesis".to_string(),
        };

        assert_eq!(candidate.id, "cand_001");
        assert!(matches!(candidate.source, DiscoverySource::Heuristic));
    }

    #[test]
    fn test_discovery_loop_step_promotes_good_candidates() {
        let evaluator = MockEvaluator::new(0.5, 0.9); // score=0.9 > threshold=0.5, passes safety
        let mut loop_instance = DiscoveryLoop::new(evaluator);

        let variant = Variant {
            id: 0,
            name: "good_variant".to_string(),
            version: "1.0".to_string(),
            variant_type: VariantType::Quantization,
            constraints: VariantConstraints::default(),
            expected_latency_us: 500,
            is_default: false,
            metadata: HashMap::new(),
        };

        let candidate = Candidate {
            id: "cand_good".to_string(),
            source: DiscoverySource::GeneticAlgorithm,
            variant,
            hypothesis: "Should be promoted".to_string(),
        };

        loop_instance.add_candidate(candidate);
        let promoted = loop_instance.step();

        assert_eq!(promoted.len(), 1);
        assert_eq!(promoted[0].name, "good_variant");
        assert!(promoted[0].metadata.contains_key("omega_score"));
    }

    #[test]
    fn test_discovery_loop_step_rejects_bad_candidates() {
        let evaluator = MockEvaluator::new(0.9, 0.5); // score=0.5 < threshold=0.9, fails safety
        let mut loop_instance = DiscoveryLoop::new(evaluator);

        let variant = Variant {
            id: 0,
            name: "bad_variant".to_string(),
            version: "1.0".to_string(),
            variant_type: VariantType::Kernel,
            constraints: VariantConstraints::default(),
            expected_latency_us: 500,
            is_default: false,
            metadata: HashMap::new(),
        };

        let candidate = Candidate {
            id: "cand_bad".to_string(),
            source: DiscoverySource::Manual,
            variant,
            hypothesis: "Should be rejected".to_string(),
        };

        loop_instance.add_candidate(candidate);
        let promoted = loop_instance.step();

        assert_eq!(promoted.len(), 0);
    }

    #[test]
    fn test_discovery_loop_empty_step() {
        let evaluator = MockEvaluator::new(0.5, 0.9);
        let mut loop_instance = DiscoveryLoop::new(evaluator);

        // No candidates added
        let promoted = loop_instance.step();
        assert_eq!(promoted.len(), 0);
    }

    #[test]
    fn test_mock_llm_observer_propose() {
        let observer = MockLlmObserver::new("gpt-4");

        let context = ObserverContext {
            recent_telemetry: vec!["Step 1: quality=0.8".to_string()],
            current_config: "{}".to_string(),
            goal_description: "Maximize stability".to_string(),
        };

        let candidate = observer.propose(&context);

        assert!(candidate.is_some());
        let cand = candidate.unwrap();
        assert!(cand.id.starts_with("cand_llm_"));
        assert!(matches!(cand.source, DiscoverySource::LlmObserver));
        assert!(!cand.hypothesis.is_empty());
    }

    #[test]
    fn test_discovery_source_variants() {
        // Ensure all variants can be constructed
        let sources = vec![
            DiscoverySource::Heuristic,
            DiscoverySource::GeneticAlgorithm,
            DiscoverySource::LlmObserver,
            DiscoverySource::GridSearch,
            DiscoverySource::Manual,
        ];

        for source in sources {
            let formatted = format!("{:?}", source);
            assert!(!formatted.is_empty());
        }
    }
}

// =============================================================================
// UNIT TESTS: Governance
// =============================================================================

mod governance_unit_tests {
    use super::*;

    #[test]
    fn test_upstream_message_proposal_serialization() {
        let msg = UpstreamMessage::Proposal {
            id: "prop_001".to_string(),
            timestamp_us: 1000,
            variant_id: Some(42),
            params: Some(HashMap::from([("lr".to_string(), 0.01)])),
            reason: "latency spike".to_string(),
            confidence: 0.95,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("prop_001"));
        assert!(json.contains("0.95"));

        let decoded: UpstreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            UpstreamMessage::Proposal { id, confidence, .. } => {
                assert_eq!(id, "prop_001");
                assert!((confidence - 0.95).abs() < 0.001);
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_upstream_message_telemetry_serialization() {
        let mut metrics = HashMap::new();
        metrics.insert("latency_p99".to_string(), 45.0);
        metrics.insert("throughput".to_string(), 1000.0);

        let msg = UpstreamMessage::Telemetry {
            timestamp_us: 2000,
            node_id: "node_alpha".to_string(),
            metrics,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("node_alpha"));

        let decoded: UpstreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            UpstreamMessage::Telemetry { node_id, .. } => {
                assert_eq!(node_id, "node_alpha");
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_upstream_message_alert_serialization() {
        let msg = UpstreamMessage::Alert {
            timestamp_us: 3000,
            severity: AlertSeverity::Critical,
            message: "Memory pressure detected".to_string(),
            context: HashMap::from([("heap_used".to_string(), "95%".to_string())]),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Critical"));

        let decoded: UpstreamMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            UpstreamMessage::Alert { severity, .. } => {
                assert_eq!(severity, AlertSeverity::Critical);
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_downstream_message_approval() {
        let msg = DownstreamMessage::Approval {
            proposal_id: "prop_001".to_string(),
            proposal_digest: "sha256:abc123".to_string(),
            rollout: Some(RolloutConfig {
                initial_percent: 0.1,
                increment_per_minute: 0.05,
                max_percent: 1.0,
            }),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("prop_001"));
        assert!(json.contains("0.1"));
    }

    #[test]
    fn test_downstream_message_enforce() {
        let actions = vec![
            EnforcementAction::SetVariant(42),
            EnforcementAction::SetParams(HashMap::from([("lr".to_string(), 0.001)])),
            EnforcementAction::EmergencyStop,
            EnforcementAction::PauseAdaptation,
            EnforcementAction::ResumeAdaptation,
        ];

        for action in actions {
            let msg = DownstreamMessage::Enforce {
                action,
                reason: "Test enforcement".to_string(),
            };
            let json = serde_json::to_string(&msg).unwrap();
            assert!(json.contains("Test enforcement"));
        }
    }

    #[test]
    fn test_alert_severity_variants() {
        let severities = vec![
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Error,
            AlertSeverity::Critical,
        ];

        for sev in severities {
            let formatted = format!("{:?}", sev);
            assert!(!formatted.is_empty());
        }
    }
}

// =============================================================================
// INTEGRATION TESTS: Omega → Catalog
// =============================================================================

mod integration_tests {
    use super::*;

    /// Helper to create a test variant
    fn create_test_variant(name: &str, variant_type: VariantType) -> Variant {
        Variant {
            id: 0,
            name: name.to_string(),
            version: "1.0".to_string(),
            variant_type,
            constraints: VariantConstraints::default(),
            expected_latency_us: 1000,
            is_default: false,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_omega_to_catalog_integration() {
        // Create a mock evaluator that always passes
        struct AlwaysPassEvaluator;
        impl Evaluator for AlwaysPassEvaluator {
            fn evaluate(&self, candidate: &Candidate) -> EvaluationResult {
                EvaluationResult {
                    candidate_id: candidate.id.clone(),
                    score: 0.95,
                    metrics: HashMap::new(),
                    passed_safety: true,
                    duration_ms: 50,
                }
            }
        }

        let mut discovery = DiscoveryLoop::new(AlwaysPassEvaluator);
        let mut catalog = VariantCatalog::new();

        // Add candidate to discovery loop
        let variant = create_test_variant("new_kernel", VariantType::Kernel);
        let candidate = Candidate {
            id: "cand_integration".to_string(),
            source: DiscoverySource::LlmObserver,
            variant,
            hypothesis: "Integration test".to_string(),
        };

        discovery.add_candidate(candidate);

        // Run discovery step
        let promoted = discovery.step();
        assert_eq!(promoted.len(), 1);

        // Add promoted variant to catalog
        for var in promoted {
            catalog.add(var);
        }

        assert_eq!(catalog.len(), 1);
        let kernels = catalog.by_type(&VariantType::Kernel);
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].name, "new_kernel");
    }

    #[test]
    fn test_catalog_to_bandit_integration() {
        let mut catalog = VariantCatalog::new();
        let mut bandit = ContextualBandit::new(BanditConfig {
            seed: 42,
            exploration_prob: 0.0, // Deterministic for testing
            ..Default::default()
        });

        // Add variants to catalog
        let v1 = create_test_variant("variant_a", VariantType::Quantization);
        let v2 = create_test_variant("variant_b", VariantType::Quantization);
        let id1 = catalog.add(v1);
        let id2 = catalog.add(v2);

        // Get eligible variants from catalog
        let eligible = catalog.all_ids();
        assert_eq!(eligible.len(), 2);

        // Bandit selects from eligible
        let selection = bandit.select(&eligible, None).unwrap();
        assert!(eligible.contains(&selection.variant_id));

        // Update bandit with reward
        bandit.update(selection.variant_id, 1.0);
        let stats = bandit.stats(selection.variant_id).unwrap();
        assert_eq!(stats.pulls, 1);
    }

    #[test]
    fn test_bandit_learns_preference() {
        let mut bandit = ContextualBandit::new(BanditConfig {
            seed: 42,
            exploration_prob: 0.0,
            ..Default::default()
        });

        let eligible = vec![1, 2, 3];

        // Train: variant 1 always succeeds, others fail
        for _ in 0..50 {
            bandit.update(1, 1.0);
            bandit.update(2, 0.0);
            bandit.update(3, 0.0);
        }

        // Count selections over 100 trials
        let mut counts = HashMap::new();
        for _ in 0..100 {
            let sel = bandit.select(&eligible, None).unwrap();
            *counts.entry(sel.variant_id).or_insert(0) += 1;
        }

        // Variant 1 should be selected most often (>80%)
        assert!(
            *counts.get(&1).unwrap_or(&0) > 80,
            "Expected variant 1 to be selected >80 times, got {:?}",
            counts
        );
    }

    #[test]
    fn test_constraint_filtering() {
        let mut catalog = VariantCatalog::new();

        // Fast variant: designed for low-latency environments (max 500µs)
        let mut fast = create_test_variant("fast", VariantType::Kernel);
        fast.constraints.max_latency_p99_us = Some(500);
        catalog.add(fast);

        // Slow variant: tolerates high latency (max 5000µs)
        let mut slow = create_test_variant("slow", VariantType::Kernel);
        slow.constraints.max_latency_p99_us = Some(5000);
        catalog.add(slow);

        // Test 1: With a 400µs latency, both variants are eligible (400 <= 500 and 400 <= 5000)
        let eligible_low = catalog.filter_eligible(
            400,     // latency budget
            1000000, // memory budget
            0.0,     // min quality
            1.0,     // cost budget
            "any",   // gpu type
        );
        assert_eq!(eligible_low.len(), 2, "Both variants should work at 400µs");

        // Test 2: With a 600µs latency, only slow variant is eligible (600 > 500, but 600 <= 5000)
        let eligible_high = catalog.filter_eligible(
            600, // latency budget (exceeds fast's max of 500)
            1000000, 0.0, 1.0, "any",
        );
        assert_eq!(
            eligible_high.len(),
            1,
            "Only slow variant should work at 600µs"
        );
        let selected = catalog.get(eligible_high[0]).unwrap();
        assert_eq!(selected.name, "slow");
    }
}

// =============================================================================
// END-TO-END TEST: Full Autopoiesis Pipeline
// =============================================================================

#[test]
fn test_full_autopoiesis_pipeline() {
    // 1. TIER Ω: Discovery generates candidates
    struct SimulatedEvaluator;
    impl Evaluator for SimulatedEvaluator {
        fn evaluate(&self, candidate: &Candidate) -> EvaluationResult {
            // Simulate: kernels pass, others fail
            let passes = matches!(candidate.variant.variant_type, VariantType::Kernel);
            EvaluationResult {
                candidate_id: candidate.id.clone(),
                score: if passes { 0.92 } else { 0.5 },
                metrics: HashMap::from([("latency".to_string(), 100.0)]),
                passed_safety: passes,
                duration_ms: 200,
            }
        }
    }

    let mut discovery = DiscoveryLoop::new(SimulatedEvaluator);

    // Add multiple candidates
    let kernel_variant = Variant {
        id: 0,
        name: "optimized_kernel".to_string(),
        version: "2.0".to_string(),
        variant_type: VariantType::Kernel,
        constraints: VariantConstraints::default(),
        expected_latency_us: 800,
        is_default: false,
        metadata: HashMap::new(),
    };
    discovery.add_candidate(Candidate {
        id: "cand_kernel".to_string(),
        source: DiscoverySource::GeneticAlgorithm,
        variant: kernel_variant,
        hypothesis: "Faster kernel".to_string(),
    });

    let adapter_variant = Variant {
        id: 0,
        name: "lora_adapter".to_string(),
        version: "1.0".to_string(),
        variant_type: VariantType::Adapter,
        constraints: VariantConstraints::default(),
        expected_latency_us: 1000,
        is_default: false,
        metadata: HashMap::new(),
    };
    discovery.add_candidate(Candidate {
        id: "cand_adapter".to_string(),
        source: DiscoverySource::Manual,
        variant: adapter_variant,
        hypothesis: "Domain-specific adapter".to_string(),
    });

    let promoted = discovery.step();
    assert_eq!(promoted.len(), 1, "Only kernel should pass");
    assert_eq!(promoted[0].name, "optimized_kernel");

    // 2. TIER 2: Add to catalog and select via bandit
    let mut catalog = VariantCatalog::new();

    // Add a default variant
    let default_variant = Variant {
        id: 0,
        name: "baseline".to_string(),
        version: "1.0".to_string(),
        variant_type: VariantType::Kernel,
        constraints: VariantConstraints::default(),
        expected_latency_us: 1200,
        is_default: true,
        metadata: HashMap::new(),
    };
    let default_id = catalog.add(default_variant);

    // Add promoted variant
    for var in promoted {
        catalog.add(var);
    }

    assert_eq!(catalog.len(), 2);

    // Bandit selection
    let mut bandit = ContextualBandit::new(BanditConfig {
        seed: 12345,
        exploration_prob: 0.1,
        ..Default::default()
    });

    let eligible = catalog.all_ids();
    let selection = bandit.select(&eligible, Some(default_id)).unwrap();

    // Selection should be one of the variants
    assert!(eligible.contains(&selection.variant_id));

    // 3. TIER 1: Governance (simulate approval)
    let proposal = UpstreamMessage::Proposal {
        id: "prop_e2e".to_string(),
        timestamp_us: 1234567890,
        variant_id: Some(selection.variant_id),
        params: None,
        reason: "Discovered via Omega tier".to_string(),
        confidence: 0.92,
    };

    let json = serde_json::to_string(&proposal).unwrap();
    assert!(json.contains("prop_e2e"));

    // Simulate downstream approval
    let approval = DownstreamMessage::Approval {
        proposal_id: "prop_e2e".to_string(),
        proposal_digest: "sha256:e2e_test".to_string(),
        rollout: Some(RolloutConfig {
            initial_percent: 0.05,
            increment_per_minute: 0.1,
            max_percent: 1.0,
        }),
    };

    let approval_json = serde_json::to_string(&approval).unwrap();
    assert!(approval_json.contains("0.05"));

    // Pipeline complete!
    println!("✅ Full Autopoiesis Pipeline Test Passed");
    println!("   - Tier Ω: 2 candidates → 1 promoted");
    println!(
        "   - Tier 2: Catalog({} variants), Bandit selected {}",
        catalog.len(),
        selection.variant_id
    );
    println!("   - Tier 1: Governance approval serialized");
}
