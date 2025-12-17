# Feature Specification: Adaptive Engine

**Feature ID**: 005-adaptive-engine  
**Version**: 1.1.0  
**Status**: Draft  
**Author**: ArqonHPO Team  
**Date**: 2025-12-16  
**Constitution Reference**: v1.3.0 (II.16-23, VIII.4-6, IX.2)

---

## 1. Executive Summary

The Adaptive Engine enables **real-time parameter tuning at microsecond latency** within live control loops. Unlike batch optimization (the existing `Solver`), the Adaptive Engine runs continuously, making small, bounded adjustments based on streaming telemetry.

This specification defines the **end-to-end SPSA handshake protocol**, enforces **hard tier boundaries via traits**, and establishes **control-theoretic safety invariants** beyond simple guardrails.

**Architecture**:
- **Tier 1 (SafeExecutor)**: Sole actuator—validates and applies changes
- **Tier 2 (AdaptiveProposer)**: Proposal generator—reads telemetry, proposes deltas
- **Tier Ω (Offline Discovery)**: Candidate generator—never in hot path

**Target latency**: T2_decision_us ≤ 1,000 µs (digest → proposal)

---

## Constitution Constraints *(from v1.3.0)*

The following non-negotiable constraints from the ArqonHPO constitution apply to this feature:

- **II.16**: SPSA MUST use ±1 Bernoulli perturbations, ChaCha8Rng, and decay schedules α=0.602, γ=0.101
- **II.17**: All updates MUST pass through SafetyExecutor; direct writes to live config are forbidden
- **II.18**: Config swaps MUST be atomic with monotonic generation counter and zero-alloc hot path
- **II.19**: Telemetry digests MUST be ≤128 bytes with mandatory timestamp_us and objective_value
- **II.20**: Tier 2 (Adaptive Engine) MUST NOT directly mutate production state
- **II.21**: Tier 2 bypassing Tier 1, or mixing tier logic, is merge-blocked
- **II.22**: Only Approved/Promoted variants are eligible for online selection
- **II.23**: Unbounded exploration and oscillation (thrashing) are forbidden
- **VIII.4**: T2_decision_us ≤ 1,000 µs, T1_apply_us ≤ 100 µs, E2E_visible_us ≤ 2,000 µs
- **VIII.5**: Audit-to-disk MUST be decoupled via ring buffer; blocking I/O in hot path is forbidden
- **IX.2**: Events MUST include correlation IDs (run_id, proposal_id, config_version)

---

## User Scenarios & Testing

### User Story 1 - Online Parameter Adaptation (Priority: P1)

An ML operator wants their inference service to automatically adapt quantization and scheduling parameters as traffic patterns change throughout the day, without manual intervention.

**Why this priority**: Core value proposition — enables autonomous optimization.

**Independent Test**: Deploy with synthetic drift; verify config converges toward optimum within SLA.

**Acceptance Scenarios**:

1. **Given** initial config and stationary objective, **When** 100 telemetry digests are processed, **Then** config converges closer to optimum with no constraint violations
2. **Given** drifting objective (optimum shifts by 20%), **When** engine detects regression, **Then** config tracks new optimum within 30 eval cycles
3. **Given** constraint_margin < 0, **When** proposal is generated, **Then** proposal prioritizes feasibility restoration

---

### User Story 2 - Safety-First Operation (Priority: P1)

A platform engineer requires that adaptation never destabilizes production, even under adversarial telemetry or misconfiguration.

**Why this priority**: Safety is non-negotiable for production systems.

**Independent Test**: Inject adversarial telemetry; verify engine enters SafeMode without applying bad configs.

**Acceptance Scenarios**:

1. **Given** telemetry with config_generation mismatch, **When** SPSA processes digests, **Then** stale digests are discarded (AC-13)
2. **Given** 5 consecutive objective regressions, **When** engine detects instability, **Then** SafeMode is entered and NoChange proposals emitted (AC-15)
3. **Given** audit queue is full, **When** new event is enqueued, **Then** adaptation halts and SAFE_MODE event is emitted without silent drop (AC-17)

---

### User Story 3 - Anti-Thrashing Under Noise (Priority: P2)

A performance engineer wants to ensure the engine doesn't oscillate parameters back and forth under noisy objectives.

**Why this priority**: Thrashing wastes resources and can destabilize downstream systems.

**Independent Test**: Inject oscillating objective; verify direction flip count is bounded.

**Acceptance Scenarios**:

1. **Given** stationary objective with high noise, **When** engine adapts for 60 seconds, **Then** direction flips per dimension ≤ 3 per minute (AC-14)
2. **Given** direction_flip_limit exceeded, **When** next proposal would flip again, **Then** cooldown is enforced and NoChange emitted

---

### User Story 4 - Audit Completeness (Priority: P2)

A compliance officer requires that every proposal, apply, and rollback is logged with no silent drops.

**Why this priority**: Auditability is constitutionally mandated.

**Independent Test**: Run 1000 adaptation cycles; verify audit log contains exactly 1000 of each event type.

**Acceptance Scenarios**:

1. **Given** proposal generated, **When** event emitted, **Then** audit queue contains event with correct correlation IDs
2. **Given** audit queue at 80% capacity, **When** new event enqueued, **Then** HighWaterMark warning is emitted
3. **Given** normal operation, **When** 1000 proposals are generated, **Then** 1000 proposal events are logged (no drops)

---

### Edge Cases

- What happens when eval_window_us expires without sufficient digests? → Emit `Proposal::NoChange(EvalTimeout)`
- How does system handle stale digests from previous config generation? → Discard with `DigestValidity::WrongGeneration`
- What if constraint_margin goes extremely negative (<-0.5)? → Emergency rollback to baseline
- What happens if SafeMode is triggered during mid-perturbation SPSA cycle? → Complete current cycle with NoChange, then freeze
- How does system handle 0 telemetry digests for extended period? → Timeout counter increments; 3 timeouts → SafeMode

---

## 2. Problem Statement


### 2.1 The Challenge

Production AI systems operate in non-stationary environments:
- Traffic patterns shift
- Hardware throttles under load
- Drift degrades model accuracy
- Constraints change over time

Traditional HPO tools assume human-time feedback loops. But if the optimization loop is fast enough (microseconds), **tuning becomes a control primitive**: safe, bounded, auditable adaptation.

### 2.2 Design Goals

| Goal | Metric |
|:---|:---|
| **Speed** | Proposal latency ≤ 1ms (p99) |
| **Safety** | All changes pass guardrails + control safety before application |
| **Determinism** | Same (seed, digest stream) → byte-identical proposals |
| **Bounded Change** | Max 10% delta per step, rate-limited, anti-thrashing |
| **Auditability** | Every proposal/apply/rollback is logged; never silently dropped |

### 2.3 Non-Goals

- **Distributed coordination**: Single-node scope (ArqonBus integration is 006-governance)
- **Variant selection**: Discrete choices are 008-variant-catalog
- **Offline discovery**: Candidate generation is 007-omega
- **Python bindings**: Defer to post-Rust-core completion

---

## 3. Architecture

### 3.1 Trait-Based Tier Boundaries

> **Critical**: Tier 2 MUST NOT have access to `AtomicConfig` or any method that mutates production state.

```rust
/// TIER 2: Proposal generation only. Cannot mutate config.
pub trait AdaptiveProposer {
    fn observe(&mut self, digest: TelemetryDigest) -> ProposalResult;
    fn current_perturbation(&self) -> Option<Perturbation>;
    fn iteration(&self) -> u64;
}

/// TIER 1: Sole actuator. Has exclusive write access to config.
pub trait SafeExecutor {
    fn apply(&mut self, proposal: Proposal) -> Result<ApplyReceipt, Violation>;
    fn rollback(&mut self) -> Result<RollbackReceipt, Violation>;
    fn set_baseline(&mut self);
    fn snapshot(&self) -> ConfigSnapshot;
}
```

**Enforcement**:
- `AdaptiveProposer` implementations MUST NOT hold a reference to `AtomicConfig`
- Only `SafeExecutor` implementations may call `AtomicConfig::swap()`
- Merge-blocking: Any code path where Tier 2 mutates config is rejected

### 3.2 Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     SPSA HANDSHAKE PROTOCOL                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────┐  observe()  ┌─────────────┐  apply(+Δ)  ┌──────────┐  │
│  │Telemetry│────────────▶│   Tier 2    │────────────▶│  Tier 1  │  │
│  │  Digest │             │  Proposer   │             │ Executor │  │
│  └─────────┘             └─────────────┘             └──────────┘  │
│       ▲                        │                          │        │
│       │                        │ Proposal::ApplyPlus      │        │
│       │                        ▼                          ▼        │
│       │                  ┌─────────────┐           ┌──────────┐   │
│       │                  │ DataPlane   │───────────│  Config  │   │
│       │                  │ runs under  │  new gen  │  Atomic  │   │
│       │                  │  +Δ config  │           │  Swap    │   │
│       │                  └─────────────┘           └──────────┘   │
│       │                        │                                   │
│       │    eval_window digests │                                   │
│       └────────────────────────┘                                   │
│                                                                     │
│  (Repeat for −Δ, then compute gradient and propose real update)    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. SPSA Handshake Protocol

> **This is the most critical section**: SPSA requires 2 evaluations under different configs.

### 4.1 Evaluation Window Contract

| Parameter | Default | Description |
|:---|:---|:---|
| `eval_window_digests` | 5 | Minimum digests to collect per perturbation |
| `eval_window_us` | 500,000 (500ms) | Maximum time to wait for eval_window_digests |
| `settle_time_us` | 10,000 (10ms) | Time after apply before digests count |

**Rules**:
- Digests with `timestamp_us < apply_timestamp_us + settle_time_us` are **discarded** (settling period)
- Digests with `config_generation != expected_generation` are **discarded** (stale)
- Objective = `aggregation(valid digests over eval window)`

### 4.2 Objective Aggregation

| Method | Use Case |
|:---|:---|
| `TrimmedMean(10%)` | Default: robust to outliers |
| `Median` | Alternative for heavy-tailed noise |
| `Mean` | Only for low-noise environments |

Configurable via `AdaptiveEngineConfig::objective_aggregation`.

### 4.3 SPSA State Machine (Complete)

```
                          ┌──────────────────────────────────────┐
                          │                                      │
                          ▼                                      │
     ┌────────┐   generate Δ   ┌───────────────────────────┐    │
     │ Ready  │───────────────▶│ ApplyingPlus              │    │
     └────────┘                │ (emit Proposal::ApplyPlus) │    │
         ▲                     └─────────────┬─────────────┘    │
         │                                   │                   │
         │                            Tier 1 applies +Δ         │
         │                                   │                   │
         │                                   ▼                   │
         │                     ┌───────────────────────────┐    │
         │                     │ WaitingPlus               │    │
         │                     │ (collect eval_window)     │    │
         │                     │ (discard stale digests)   │    │
         │                     └─────────────┬─────────────┘    │
         │                                   │                   │
         │                      eval_window complete            │
         │                           y+ = aggregate()           │
         │                                   │                   │
         │                                   ▼                   │
         │                     ┌───────────────────────────┐    │
         │                     │ ApplyingMinus             │    │
         │                     │ (emit Proposal::ApplyMinus)│    │
         │                     └─────────────┬─────────────┘    │
         │                                   │                   │
         │                            Tier 1 applies −Δ         │
         │                                   │                   │
         │                                   ▼                   │
         │                     ┌───────────────────────────┐    │
         │                     │ WaitingMinus              │    │
         │                     │ (collect eval_window)     │    │
         │                     └─────────────┬─────────────┘    │
         │                                   │                   │
         │                      eval_window complete            │
         │                           y- = aggregate()           │
         │                                   │                   │
         │                                   ▼                   │
         │                     ┌───────────────────────────┐    │
         │   k++               │ ComputeGradient           │────┘
         │                     │ gradient = (y+ - y-)/(2Δ) │
         │                     │ emit Proposal::Update     │
         └─────────────────────│ (real delta = -a_k * grad)│
                               └───────────────────────────┘
```

### 4.4 Timeout & Error Handling

| Condition | Action |
|:---|:---|
| `eval_window_us` elapsed without sufficient digests | Emit `Proposal::NoChange`, increment `eval_timeout_count`, stay in current state |
| 3 consecutive timeouts | Enter `SafeMode`, emit `SafeModeEntered` event |
| Tier 1 rejects proposal (Violation) | Log violation, do NOT increment k, retry or enter SafeMode |

### 4.5 Proposal Types

```rust
pub enum Proposal {
    /// Apply +Δ perturbation for y+ evaluation
    ApplyPlus { 
        perturbation_id: u64,
        delta: ParamVec,
    },
    /// Apply −Δ perturbation for y− evaluation  
    ApplyMinus { 
        perturbation_id: u64,
        delta: ParamVec,
    },
    /// Apply real gradient-based update
    Update {
        iteration: u64,
        delta: ParamVec,
        gradient_estimate: ParamVec,
    },
    /// No change (timeout, insufficient data, safe mode)
    NoChange {
        reason: NoChangeReason,
    },
}

pub enum NoChangeReason {
    EvalTimeout,
    SafeMode,
    ConstraintViolation,
    CooldownActive,
}
```

---

## 5. Control Safety Invariants

> Beyond guardrails: these prevent oscillation and instability even when individual deltas are "legal."

### 5.1 Anti-Thrashing Rules

| Invariant | Default | Description |
|:---|:---|:---|
| `direction_flip_limit` | 3 per dimension | Max sign changes per dimension per minute |
| `cooldown_after_flip_us` | 30,000,000 (30s) | Mandatory cooldown after hitting flip limit |
| `hysteresis_threshold` | 0.1 | Don't change direction unless gradient > threshold |

**Violation**: `Thrashing` → enter SafeMode for `cooldown_after_flip_us`

### 5.2 Cumulative Change Budget

| Invariant | Default | Description |
|:---|:---|:---|
| `max_cumulative_delta_per_minute` | 0.5 (50%) | Total movement allowed per dimension per minute |
| `budget_window_us` | 60,000,000 (60s) | Rolling window for budget |

**Violation**: `BudgetExhausted` → emit NoChange until budget replenishes

### 5.3 Stop-on-Instability

| Invariant | Default | Description |
|:---|:---|:---|
| `regression_count_limit` | 5 | Consecutive eval cycles where objective worsens |
| `regression_threshold` | 0.01 | Minimum worsening to count as regression |

**Violation**: `ObjectiveRegression` → freeze adaptation, emit SafeModeEntered

### 5.4 Constraint-First Policy

When `constraint_margin < 0` (constraints violated):
1. **Prioritize feasibility**: Proposals MUST move toward feasibility
2. **No optimization**: Do not optimize objective until `constraint_margin >= 0`
3. **Emergency rollback**: If constraint margin < -0.5, trigger immediate rollback

### 5.5 Safe Mode Latch

```rust
pub struct SafeMode {
    pub entered_at_us: u64,
    pub reason: SafeModeReason,
    pub exit_condition: SafeModeExit,
}

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

**Rules**:
- SafeMode emits `Proposal::NoChange` until exit condition met
- Every SafeMode entry emits `SafeModeEntered` event
- Exit requires explicit acknowledgment for `ManualReset`

---

## 6. Telemetry & Config Correlation

> Without this, SPSA mixes signals from old configs → phantom gradients.

### 6.1 TelemetryDigest Schema (Extended)

```rust
#[derive(Clone)]
pub struct TelemetryDigest {
    // Required
    pub timestamp_us: u64,
    pub objective_value: f64,
    
    // Config correlation (MANDATORY)
    pub config_generation: u64,  // Generation observed by dataplane at emit time
    
    // Optional metrics
    pub latency_p99_us: Option<u64>,
    pub throughput_rps: Option<f64>,
    pub error_rate: Option<f64>,
    pub constraint_margin: Option<f64>,
}
```

### 6.2 Staleness Rules

| Rule | Implementation |
|:---|:---|
| Config mismatch | Discard if `digest.config_generation != expected_generation` |
| Pre-settle | Discard if `digest.timestamp_us < apply_timestamp_us + settle_time_us` |
| Too old | Discard if `digest.timestamp_us < now_us - max_digest_age_us` |

**Validation**: `validate_digest()` returns `DigestValidity::{Valid, Stale, WrongGeneration, TooOld}`

### 6.3 Attribution Invariant

> **AC-13**: SPSA MUST NOT use objective values from digests whose `config_generation` doesn't match the applied perturbation's generation.

---

## 7. Internal Representation (Performance)

> HashMap<String, f64> is a performance trap in microsecond budgets.

### 7.1 ParamId + Dense Array

```rust
/// Stable parameter identifier (u16 = up to 65K params)
pub type ParamId = u16;

/// Mapping from string names to stable IDs (setup-time only)
pub struct ParamRegistry {
    name_to_id: HashMap<String, ParamId>,
    id_to_name: Vec<String>,
}

/// Dense parameter vector (hot path)
#[derive(Clone)]
pub struct ParamVec {
    values: SmallVec<[f64; 16]>,  // Stack for ≤16 params
}

/// ConfigSnapshot with dense representation
pub struct ConfigSnapshot {
    pub params: ParamVec,
    pub generation: u64,
}
```

### 7.2 Boundary Conversion

```rust
impl ParamRegistry {
    /// Setup-time: parse config strings → ParamVec
    pub fn to_param_vec(&self, map: &HashMap<String, f64>) -> ParamVec;
    
    /// Serialization: ParamVec → HashMap for artifacts/API
    pub fn to_map(&self, vec: &ParamVec) -> HashMap<String, f64>;
}
```

**Rules**:
- Core loop uses `ParamVec` exclusively
- `HashMap<String, f64>` only at boundaries (config parsing, artifact serialization)
- Deterministic ordering: sorted by `ParamId` for reproducibility

---

## 8. Ring Buffer Contract

> VecDeque can allocate on growth. Spec must be stricter.

### 8.1 No-Allocation Guarantees

```rust
pub struct TelemetryRingBuffer {
    buffer: Box<[Option<TelemetryDigest>]>,  // Fixed allocation
    capacity: usize,
    head: usize,
    len: usize,
    drop_count: u64,  // Counter for dropped digests
}
```

| Guarantee | Implementation |
|:---|:---|
| Fixed capacity | `Box<[T; N]>` allocated once at init |
| No alloc after init | `push()` overwrites slot, never grows |
| O(1) push | Circular buffer, modulo indexing |
| Overflow tracking | `drop_count` metric exposed |

### 8.2 Overflow Policy

| Policy | Behavior |
|:---|:---|
| `DropOldest` (default) | Overwrite oldest, increment `drop_count` |
| `SignalBackpressure` | Return error, let caller decide |

**Never**: Silent drop without incrementing counter.

---

## 9. Audit Queue Policy

> Audit is sacred. Never silently drop. Never block hot path.

### 9.1 Non-Blocking Audit Contract

```rust
pub struct AuditQueue {
    queue: ArrayQueue<AuditEvent>,  // Lock-free MPSC
    capacity: usize,
    high_water_mark: usize,         // 80% capacity
}

pub enum EnqueueResult {
    Ok,
    HighWaterMark,  // Queue > 80% full, warning
    Full,           // Queue 100% full, enter SafeMode
}
```

### 9.2 Queue-Full Behavior

1. **DO NOT block** the apply path
2. **DO NOT silently drop** audit events
3. **Enter SafeMode** immediately
4. **Emit** `SAFE_MODE` event (to telemetry, not audit queue)
5. **Resume** when queue drains below high_water_mark

**AC-17**: When audit queue is full, adaptation halts and emits `SAFE_MODE` event; it never silently drops.

---

## 10. Homeostatic Regime Cache (Contract)

> Define the shape now to prevent architecture rewrites later.

### 10.1 Stable Regime Definition

A regime is **homeostatic** when over a `stability_window`:
- Objective variance < `variance_threshold`
- No constraint violations
- No SafeMode entries
- Direction flips < `flip_threshold`

### 10.2 Cache Schema

```rust
pub struct HomeostasisEntry {
    pub id: u64,
    pub config: ParamVec,
    pub context_fingerprint: u64,  // Hash of context features
    pub created_at_us: u64,
    pub stability_score: f64,
    pub constraints_satisfied: bool,
}

pub struct HomeostasisCache {
    entries: LruCache<u64, HomeostasisEntry>,
    capacity: usize,  // Default: 32
}
```

### 10.3 Re-Entry Protocol

1. Compute `context_fingerprint` from current conditions
2. Find `nearest_neighbor(fingerprint)` in cache
3. If found and `similarity > threshold`:
   - Load cached config as **proposal** (not direct apply)
   - Still passes through Tier 1 guardrails
   - Canary period before confirming re-entry
4. If not found: normal SPSA adaptation

**Implementation**: Defer to post-005, but schema and protocol are fixed.

---

## 11. Component Specifications

### 11.1 SPSA Optimizer (II.16)

| Property | Specification |
|:---|:---|
| Evaluations per update | **2** (regardless of dimension) |
| Perturbation distribution | ±1 Bernoulli (symmetric) |
| PRNG | `ChaCha8Rng::seed_from_u64(seed)` |
| Learning rate decay | `a_k = a₀ / (k + 1 + A)^α` where α = 0.602 |
| Perturbation decay | `c_k = c₀ / (k + 1)^γ` where γ = 0.101 |

**Banned Patterns**:
- ❌ Finite-difference gradients (O(n) evals)
- ❌ Gaussian perturbations (heavy tails)
- ❌ Unbounded learning rates

### 11.2 Safety Executor (II.17)

| Guardrail | Default | 
|:---|:---|
| `max_delta_per_step` | 0.1 (10%) |
| `max_updates_per_second` | 10.0 |
| `min_interval_us` | 100,000 (100ms) |

**Violation Types**: `DeltaTooLarge`, `RateLimitExceeded`, `OutOfBounds`, `UnknownParameter`, `Thrashing`, `BudgetExhausted`, `ObjectiveRegression`, `ConstraintViolation`, `AuditQueueFull`

### 11.3 Atomic Configuration (II.18)

| Requirement | Implementation |
|:---|:---|
| Atomicity | `arc_swap::ArcSwap` or `RwLock<Arc<_>>` |
| Generation counter | `AtomicU64`, monotonically increasing |
| Zero-alloc hot path | `snapshot()` = Arc clone only |
| Thread safety | Explicit `Send + Sync` bounds |

### 11.4 Configuration

```rust
pub struct AdaptiveEngineConfig {
    // SPSA
    pub seed: u64,
    pub learning_rate: f64,
    pub perturbation_scale: f64,
    
    // Evaluation
    pub eval_window_digests: usize,
    pub eval_window_us: u64,
    pub settle_time_us: u64,
    pub objective_aggregation: AggregationMethod,
    
    // Control Safety
    pub direction_flip_limit: u32,
    pub cooldown_after_flip_us: u64,
    pub max_cumulative_delta_per_minute: f64,
    pub regression_count_limit: u32,
    
    // Guardrails
    pub guardrails: Guardrails,
    
    // Buffers
    pub telemetry_buffer_capacity: usize,
    pub audit_queue_capacity: usize,
}
```

---

## 12. Timing Contracts (VIII.4-6)

| Window | Definition | Budget |
|:---|:---|:---|
| `T2_decision_us` | Digest validated → proposal emitted | ≤ 1,000 µs |
| `T1_apply_us` | Proposal received → atomic swap done | ≤ 100 µs |
| `E2E_visible_us` | Digest available → dataplane sees config | ≤ 2,000 µs |

**Hot Path Constraints**:
- No heap allocations in T1 window
- No blocking syscalls
- No HashMap operations in core loop
- Audit via lock-free queue, async flush

---

## 13. Structured Events (IX.2)

| Event | Trigger | Key Fields |
|:---|:---|:---|
| `digest` | Telemetry pushed | `config_generation`, `validity` |
| `proposal` | Tier 2 emits | `proposal_type`, `perturbation_id` |
| `apply` | Tier 1 applies | `new_generation`, `apply_latency_us` |
| `rollback` | Tier 1 rollback | `reason`, `reverted_to_generation` |
| `safe_mode_entered` | SafeMode triggered | `reason`, `exit_condition` |
| `safe_mode_exited` | SafeMode cleared | `duration_us`, `exit_reason` |

---

## 14. Acceptance Criteria

### 14.1 Functional (Existing)

| ID | Criterion |
|:---|:---|
| AC-1 | SPSA produces identical perturbations for same (seed, k) |
| AC-2 | SPSA returns proposal after exactly 2 eval windows |
| AC-3 | SafetyExecutor rejects delta > max_delta |
| AC-4 | SafetyExecutor rejects out-of-bounds |
| AC-5 | SafetyExecutor rejects unknown parameter |
| AC-6 | AtomicConfig generation counter is monotonic |
| AC-7 | Rollback restores baseline exactly |
| AC-8 | Ring buffer evicts oldest on overflow |

### 14.2 Non-Functional (Existing)

| ID | Criterion |
|:---|:---|
| AC-9 | `TelemetryDigest` ≤ 128 bytes |
| AC-10 | `AtomicConfig` is `Send + Sync` |
| AC-11 | `T2_decision_us` ≤ 1,000 µs (p99) |
| AC-12 | Zero allocations in apply path |

### 14.3 Critical Invariants (NEW)

| ID | Criterion |
|:---|:---|
| **AC-13** | **Config attribution**: SPSA MUST NOT use objective values from digests whose `config_generation` doesn't match applied perturbation |
| **AC-14** | **No thrashing**: Under stationary objective, direction flips ≤ `direction_flip_limit` per minute per dimension |
| **AC-15** | **Stop-on-instability**: After `regression_count_limit` consecutive regressions, engine emits `NoChange` until reset/cooldown |
| **AC-16** | **Constraint-first**: When `constraint_margin < 0`, proposals prioritize feasibility; no "optimize into violation" |
| **AC-17** | **Audit queue pressure**: When audit queue full, adaptation halts + emits `SAFE_MODE`; never silently drops |
| **AC-18** | **Deterministic serialization**: Same seed + same digest stream → byte-identical proposal artifacts |

---

## 15. Verification Plan

### 15.1 Unit Tests

```bash
cargo test -p arqonhpo-core adaptive_engine
```

Expected: 40+ tests covering all components and invariants.

### 15.2 Property-Based Tests

- SPSA: perturbation symmetry
- SafetyExecutor: valid deltas pass, invalid fail
- Config attribution: never mix generations
- Anti-thrashing: flip count bounded

### 15.3 Benchmark Regression

CI fails if:
- `T2_decision_us` p99 > 1,000 µs (release)
- `T1_apply_us` p99 > 100 µs (release)
- Any allocation detected in hot path

### 15.4 Integration Tests

1. **Convergence under stationary objective**: Engine converges to optimum
2. **Stability under drift**: No oscillation when optimum shifts slowly
3. **SafeMode entry**: Triggers correctly on thrashing/regression
4. **Config attribution**: Simulated stale digests are discarded
5. **Audit completeness**: All events logged, none dropped

---

## 16. File Structure

```
crates/core/src/
├── adaptive_engine/
│   ├── mod.rs              # Re-exports, AdaptiveEngineConfig
│   ├── proposer.rs         # AdaptiveProposer trait + Spsa impl
│   ├── executor.rs         # SafeExecutor trait + SafetyExecutor impl
│   ├── spsa.rs             # SPSA algorithm, state machine
│   ├── control_safety.rs   # Anti-thrashing, budget, regression detection
│   ├── config_atomic.rs    # AtomicConfig, ConfigSnapshot, ParamVec
│   ├── telemetry.rs        # TelemetryDigest, TelemetryRingBuffer
│   ├── audit.rs            # AuditQueue, audit events
│   └── homeostasis.rs      # HomeostasisCache (stub for now)
└── lib.rs
```

---

## 17. Dependencies

| Crate | Purpose |
|:---|:---|
| `rand_chacha` | ChaCha8Rng |
| `rand` | RNG traits, Bernoulli |
| `smallvec` | Stack-allocated ParamVec |
| `arc_swap` | Lock-free atomic config (optional) |
| `crossbeam` | Lock-free audit queue |

---

## 18. Risks & Mitigations

| Risk | Mitigation |
|:---|:---|
| RwLock contention | Use arc_swap or benchmark first |
| ParamVec SmallVec spill | Ensure ≤16 params covers 99% of cases |
| Audit queue sizing | Monitor high_water_mark, alert before full |
| SPSA local minima | Tier Ω discovery for global search |

---

## Appendix A: Constitution Cross-Reference

| Constitution | Spec Section |
|:---|:---|
| II.16 SPSA Specification | §4, §11.1 |
| II.17 Safety Executor | §5, §11.2 |
| II.18 Atomic Configuration | §7, §11.3 |
| II.19 Telemetry Digest | §6, §8 |
| II.20-21 Tier Architecture | §3 |
| II.22 Variant Catalog | Out of scope (008) |
| II.23 Safety Semantics | §5 |
| VIII.4-6 Timing/Audit | §12, §9 |
| IX.2 Structured Events | §13 |

---

## Appendix B: Spec Version History

| Version | Date | Changes |
|:---|:---|:---|
| 1.0.0 | 2025-12-16 | Initial draft |
| 1.1.0 | 2025-12-16 | Added: SPSA handshake protocol, trait-based tier boundaries, control safety invariants, config correlation, no-alloc ring buffer, ParamId internal representation, audit queue policy, homeostasis contract, 6 killer acceptance criteria |
