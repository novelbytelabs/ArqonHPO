//! Latency benchmarks for Adaptive Engine.
//!
//! Constitution: VIII.4 - Timing Contracts
//! - T2_decision_us ≤ 1,000 µs (p99)
//! - T1_apply_us ≤ 100 µs (p99)
//! - E2E_visible_us ≤ 2,000 µs (p99)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use arqonhpo_core::adaptive_engine::{
    AdaptiveEngine, AdaptiveEngineConfig, TelemetryDigest,
    Proposal, SafeExecutor, SafetyExecutor, AtomicConfig, Guardrails,
    param_vec,
};
use std::sync::Arc;

/// Benchmark T2 decision latency (observe → proposal).
fn bench_t2_decision(c: &mut Criterion) {
    let mut group = c.benchmark_group("T2_decision");
    
    for num_params in [1, 4, 16].iter() {
        let config = AdaptiveEngineConfig::default();
        let params = param_vec(&vec![0.5; *num_params]);
        let mut engine = AdaptiveEngine::new(config, params);
        
        // Pre-warm: trigger first observe to move to WaitingPlus state
        let digest = TelemetryDigest::new(1000, 0.5, 0);
        let _ = engine.observe(digest);
        
        group.bench_with_input(
            BenchmarkId::new("observe", num_params),
            num_params,
            |b, _| {
                b.iter(|| {
                    let digest = TelemetryDigest::new(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as u64,
                        black_box(0.5 + (std::time::SystemTime::now().elapsed().unwrap_or_default().as_nanos() as f64) * 1e-12),
                        0,
                    );
                    black_box(engine.observe(digest))
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark T1 apply latency (proposal → config update).
fn bench_t1_apply(c: &mut Criterion) {
    let mut group = c.benchmark_group("T1_apply");
    
    for num_params in [1, 4, 16].iter() {
        let params = param_vec(&vec![0.5; *num_params]);
        let config = Arc::new(AtomicConfig::new(params.clone()));
        let mut executor = SafetyExecutor::new(config, Guardrails::default());
        
        let delta = param_vec(&vec![0.01; *num_params]);
        let proposal = Proposal::Update {
            iteration: 0,
            delta: delta.clone(),
            gradient_estimate: delta,
        };
        
        group.bench_with_input(
            BenchmarkId::new("apply", num_params),
            num_params,
            |b, _| {
                b.iter(|| {
                    // Create a fresh proposal each time (cloning is cheap)
                    let p = proposal.clone();
                    black_box(executor.apply(p))
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark snapshot access (should be near-instant with Arc clone).
fn bench_snapshot(c: &mut Criterion) {
    let params = param_vec(&vec![0.5; 16]);
    let config = Arc::new(AtomicConfig::new(params));
    
    c.bench_function("snapshot_16params", |b| {
        b.iter(|| black_box(config.snapshot()))
    });
}

/// Benchmark SPSA perturbation generation.
fn bench_perturbation_generation(c: &mut Criterion) {
    use arqonhpo_core::adaptive_engine::{Spsa, SpsaConfig};
    
    let mut group = c.benchmark_group("SPSA");
    
    for num_params in [1, 4, 16, 64].iter() {
        let mut spsa = Spsa::new(42, *num_params, 0.1, 0.01, SpsaConfig::default());
        
        group.bench_with_input(
            BenchmarkId::new("generate_perturbation", num_params),
            num_params,
            |b, _| {
                b.iter(|| black_box(spsa.generate_perturbation()))
            },
        );
    }
    
    group.finish();
}

/// Benchmark telemetry ring buffer push.
fn bench_telemetry_buffer(c: &mut Criterion) {
    use arqonhpo_core::adaptive_engine::TelemetryRingBuffer;
    
    let mut buffer = TelemetryRingBuffer::new(1024);
    
    c.bench_function("telemetry_push", |b| {
        b.iter(|| {
            let digest = TelemetryDigest::new(1000, black_box(0.5), 0);
            buffer.push(digest);
        })
    });
}

/// Benchmark audit queue enqueue (lock-free).
fn bench_audit_queue(c: &mut Criterion) {
    use arqonhpo_core::adaptive_engine::{AuditQueue, AuditEvent, EventType};
    
    let queue = AuditQueue::new(4096);
    
    c.bench_function("audit_enqueue", |b| {
        b.iter(|| {
            let event = AuditEvent::new(EventType::Digest, black_box(1000), 1, 1);
            black_box(queue.enqueue(event))
        })
    });
}

/// Test queue saturation: enqueue never blocks, drop counter works, T1/T2 stable.
fn bench_queue_saturation(c: &mut Criterion) {
    use arqonhpo_core::adaptive_engine::{AuditQueue, AuditEvent, EventType, EnqueueResult};
    use std::time::Instant;
    
    let mut group = c.benchmark_group("Queue_Saturation");
    
    // Pre-saturate queue
    let queue = AuditQueue::new(100);
    for i in 0..100 {
        queue.enqueue(AuditEvent::new(EventType::Digest, i, 1, 1));
    }
    assert_eq!(queue.len(), 100, "Queue should be full");
    
    // Benchmark enqueue on saturated queue (should be non-blocking)
    group.bench_function("enqueue_saturated", |b| {
        b.iter(|| {
            let event = AuditEvent::new(EventType::Digest, black_box(1000), 1, 1);
            let result = queue.enqueue(event);
            assert_eq!(result, EnqueueResult::Full);
            black_box(result)
        })
    });
    
    // Verify drop counter increments (run 1000 enqueues and check)
    let initial_drops = queue.drop_count();
    let start = Instant::now();
    for _ in 0..1000 {
        let event = AuditEvent::new(EventType::Digest, 999, 1, 1);
        let _ = queue.enqueue(event);
    }
    let elapsed = start.elapsed();
    let final_drops = queue.drop_count();
    
    // Assert: 1000 drops should have been counted
    assert_eq!(final_drops - initial_drops, 1000, "All 1000 drops should be counted");
    
    // Assert: 1000 enqueues took less than 1ms (non-blocking)
    assert!(elapsed.as_micros() < 1000, "1000 saturated enqueues took {}µs, should be <1000µs", elapsed.as_micros());
    
    group.finish();
}

/// Benchmark T1 apply latency under queue saturation.
fn bench_t1_under_saturation(c: &mut Criterion) {
    use arqonhpo_core::adaptive_engine::{AuditQueue, AuditEvent, EventType};
    
    let mut group = c.benchmark_group("T1_Saturated");
    
    // Saturate a queue
    let queue = AuditQueue::new(10);
    for i in 0..10 {
        queue.enqueue(AuditEvent::new(EventType::Digest, i, 1, 1));
    }
    
    // Setup executor
    let params = param_vec(&vec![0.5; 4]);
    let config = Arc::new(AtomicConfig::new(params.clone()));
    let mut executor = SafetyExecutor::new(config, Guardrails::default());
    
    let delta = param_vec(&vec![0.01; 4]);
    let proposal = Proposal::Update {
        iteration: 0,
        delta: delta.clone(),
        gradient_estimate: delta,
    };
    
    group.bench_function("apply_4params", |b| {
        b.iter(|| {
            // Attempt enqueue on saturated queue (fast, non-blocking)
            let event = AuditEvent::new(EventType::Apply, 1000, 1, 1);
            let _ = queue.enqueue(event);
            
            // Apply proposal
            let p = proposal.clone();
            black_box(executor.apply(p))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_t2_decision,
    bench_t1_apply,
    bench_snapshot,
    bench_perturbation_generation,
    bench_telemetry_buffer,
    bench_audit_queue,
    bench_queue_saturation,
    bench_t1_under_saturation,
);

criterion_main!(benches);

