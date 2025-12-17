# Quickstart: Adaptive Engine

**Feature**: 005-adaptive-engine  
**Date**: 2025-12-16

## Overview

The Adaptive Engine provides real-time parameter tuning at microsecond latency for live control loops.

---

## Basic Usage

```rust
use arqonhpo_core::adaptive_engine::{
    AdaptiveEngine, AdaptiveEngineConfig, Guardrails,
    TelemetryDigest, Proposal,
};

// 1. Configure the engine
let config = AdaptiveEngineConfig {
    seed: 42,
    learning_rate: 0.1,
    perturbation_scale: 0.01,
    eval_window_digests: 5,
    eval_window_us: 500_000,
    settle_time_us: 10_000,
    objective_aggregation: AggregationMethod::TrimmedMean { trim_percent: 0.1 },
    guardrails: Guardrails::default(),
    telemetry_buffer_capacity: 1024,
    audit_queue_capacity: 4096,
};

// 2. Initialize with starting parameters
let initial_params = ParamVec::from([0.5, 0.3, 0.2]);
let mut engine = AdaptiveEngine::new(config, initial_params);

// 3. Set baseline for rollback
engine.set_baseline();

// 4. Main adaptation loop
loop {
    // Receive telemetry from data plane
    let digest = TelemetryDigest {
        timestamp_us: get_timestamp_us(),
        objective_value: measure_objective(),
        config_generation: engine.current().generation,
        constraint_margin: Some(measure_constraints()),
        ..Default::default()
    };
    
    // Generate proposal (Tier 2)
    match engine.observe(digest) {
        Ok(Proposal::Update { delta, .. }) => {
            // Apply through safety guardrails (Tier 1)
            match engine.apply(Proposal::Update { delta, .. }) {
                Ok(receipt) => log_apply(receipt),
                Err(violation) => handle_violation(violation),
            }
        }
        Ok(Proposal::NoChange { reason }) => {
            // Engine decided not to change
            log_no_change(reason);
        }
        Ok(Proposal::ApplyPlus { delta, .. }) |
        Ok(Proposal::ApplyMinus { delta, .. }) => {
            // SPSA perturbation phase
            engine.apply(...);
        }
        Err(e) => handle_error(e),
    }
}
```

---

## Key Concepts

### Tier Separation

```
Tier 2 (Proposer)          Tier 1 (Executor)
      │                          │
      │  observe(digest)         │
      ├─────────────────────────>│
      │                          │
      │  Proposal                │
      │<─────────────────────────┤
      │                          │
      │        apply(proposal)   │
      ├─────────────────────────>│
      │                          │
      │  Result<Receipt,Violation>
      │<─────────────────────────┤
```

**Critical**: Tier 2 cannot directly mutate config. All changes go through Tier 1's `apply()`.

### SPSA Handshake

SPSA requires 2 evaluations per update:
1. Apply +Δ → collect `eval_window` digests → compute y+
2. Apply −Δ → collect `eval_window` digests → compute y−
3. Compute gradient = (y+ - y−) / (2Δ)
4. Apply real update

### Control Safety

Beyond basic guardrails, the engine enforces:
- **Anti-thrashing**: Max 3 direction flips per dimension per minute
- **Stop-on-instability**: SafeMode after 5 consecutive regressions
- **Constraint-first**: Prioritize feasibility when constraints violated
- **Budget limits**: Max 50% cumulative change per minute

---

## Configuration Reference

| Parameter | Default | Description |
|:---|:---|:---|
| `eval_window_digests` | 5 | Min digests per SPSA perturbation |
| `eval_window_us` | 500,000 | Max time to wait for digests |
| `settle_time_us` | 10,000 | Ignore digests after config change |
| `direction_flip_limit` | 3 | Max sign changes per min per param |
| `regression_count_limit` | 5 | Consecutive worsening → SafeMode |

---

## Error Handling

```rust
match engine.apply(proposal) {
    Ok(receipt) => { /* success */ }
    Err(Violation::DeltaTooLarge { .. }) => { /* reduce step */ }
    Err(Violation::Thrashing { .. }) => { /* cooldown active */ }
    Err(Violation::AuditQueueFull) => { /* entered SafeMode */ }
    Err(other) => { /* handle other violations */ }
}
```

---

## Next Steps

- See [spec.md](spec.md) for full specification
- See [data-model.md](data-model.md) for type definitions
- See [contracts/traits.rs](contracts/traits.rs) for API contracts
