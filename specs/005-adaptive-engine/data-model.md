# Data Model: Adaptive Engine

**Feature**: 005-adaptive-engine  
**Date**: 2025-12-16

## Overview

This document defines the core data structures for the Adaptive Engine. All types are Rust structs/enums designed for microsecond-latency operation.

---

## Core Entities

### ParamRegistry

Mapping between human-readable parameter names and dense IDs.

| Field | Type | Description |
|:---|:---|:---|
| `name_to_id` | `HashMap<String, ParamId>` | Lookup by name (setup-time only) |
| `id_to_name` | `Vec<String>` | Lookup by ID |

**Invariants**:
- Created once at initialization
- Immutable after creation
- `ParamId` is `u16` (max 65K params)

---

### ParamVec

Dense parameter vector for hot-path operations.

| Field | Type | Description |
|:---|:---|:---|
| `values` | `SmallVec<[f64; 16]>` | Stack-allocated for ≤16 params |

**Invariants**:
- Order matches `ParamRegistry` order
- Zero-copy clone (SmallVec is Copy for small sizes)

---

### ConfigSnapshot

Immutable view of current configuration.

| Field | Type | Description |
|:---|:---|:---|
| `params` | `ParamVec` | Parameter values |
| `generation` | `u64` | Monotonically increasing version |

**Invariants**:
- Generation NEVER decreases
- Immutable after creation (Arc-wrapped)

---

### TelemetryDigest

Compact telemetry from data plane.

| Field | Type | Required |
|:---|:---|:---|
| `timestamp_us` | `u64` | ✓ |
| `objective_value` | `f64` | ✓ |
| `config_generation` | `u64` | ✓ (for correlation) |
| `latency_p99_us` | `Option<u64>` | |
| `throughput_rps` | `Option<f64>` | |
| `error_rate` | `Option<f64>` | |
| `constraint_margin` | `Option<f64>` | |

**Invariants**:
- `size_of::<TelemetryDigest>() <= 128`
- `config_generation` matches the config observed by emitter

---

### Proposal

Output of Tier 2 proposer.

```rust
pub enum Proposal {
    ApplyPlus { perturbation_id: u64, delta: ParamVec },
    ApplyMinus { perturbation_id: u64, delta: ParamVec },
    Update { iteration: u64, delta: ParamVec, gradient_estimate: ParamVec },
    NoChange { reason: NoChangeReason },
}

pub enum NoChangeReason {
    EvalTimeout,
    SafeMode,
    ConstraintViolation,
    CooldownActive,
    BudgetExhausted,
}
```

---

### Violation

Safety error preventing an apply.

```rust
pub enum Violation {
    DeltaTooLarge { param_id: ParamId, delta: f64, max: f64 },
    RateLimitExceeded { rate: f64, max: f64 },
    OutOfBounds { param_id: ParamId, value: f64, min: f64, max: f64 },
    UnknownParameter { param_id: ParamId },
    Thrashing { param_id: ParamId, flips: u32, limit: u32 },
    BudgetExhausted { used: f64, limit: f64 },
    ObjectiveRegression { count: u32, limit: u32 },
    ConstraintViolation { margin: f64 },
    AuditQueueFull,
}
```

---

### SafeMode

Latch state when adaptation is frozen.

| Field | Type | Description |
|:---|:---|:---|
| `entered_at_us` | `u64` | Timestamp of entry |
| `reason` | `SafeModeReason` | Why we entered |
| `exit_condition` | `SafeModeExit` | How to exit |

```rust
pub enum SafeModeReason {
    Thrashing,
    BudgetExhausted,
    ObjectiveRegression,
    AuditQueueFull,
    RepeatedViolations,
    ManualTrigger,
}

pub enum SafeModeExit {
    Timer { remaining_us: u64 },
    ManualReset,
    ObjectiveRecovery { required_improvement: f64 },
}
```

---

### Guardrails

Configuration for safety limits.

| Field | Type | Default |
|:---|:---|:---|
| `max_delta_per_step` | `f64` | 0.1 |
| `max_updates_per_second` | `f64` | 10.0 |
| `min_interval_us` | `u64` | 100,000 |
| `direction_flip_limit` | `u32` | 3 |
| `cooldown_after_flip_us` | `u64` | 30,000,000 |
| `max_cumulative_delta_per_minute` | `f64` | 0.5 |
| `regression_count_limit` | `u32` | 5 |

---

### AuditEvent

Structured event for logging.

| Field | Type | Description |
|:---|:---|:---|
| `event_type` | `EventType` | digest/proposal/apply/rollback/safe_mode |
| `timestamp_us` | `u64` | Event timestamp |
| `run_id` | `u64` | Correlation ID |
| `proposal_id` | `Option<u64>` | If applicable |
| `config_version` | `u64` | Current generation |
| `payload` | `EventPayload` | Event-specific data |

---

## State Transitions

### SPSA State Machine

```
Ready → ApplyingPlus → WaitingPlus → ApplyingMinus → WaitingMinus → ComputeGradient → Ready
```

### SafeMode Transitions

```
Normal → [Violation] → SafeMode → [Exit Condition] → Normal
```
