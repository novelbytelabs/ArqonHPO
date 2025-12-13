<!--
Sync Impact Report:

- ArqonHPO Constitution v1.0.0 (2025-12-13)
-->

# ArqonHPO Constitution

**Version**: 1.0.0  
**Ratification Date**: 2025-12-13  
**Last Amended**: 2025-12-13  

This document defines the **non-negotiable principles** that govern how ArqonHPO is designed, evolved, and maintained.

It exists to protect ArqonHPO from accidental bloat, regression, silent breakage, and “clever” shortcuts that erode trust.

If a decision conflicts with this constitution, **the decision is wrong**.

> **Spec Kit Note:** This constitution is the hard sandbox for all `/speckit.*` commands.  
> Specs, plans, and tasks **must not** violate the constraints in Sections II–XI.

---

# ULTIMATE INTEGRITY COVENANT

This constitution is a **hard constraint** on all engineering work. It exists to prevent the failure modes that destroy real systems:

- “Happy path” engineering that collapses under real inputs
- Pseudocode / placeholders / stubs presented as completion
- Tests that “check boxes” but don’t model production reality
- Fake evidence (invented logs, benchmarks, screenshots, coverage, results)
- Unnamed technical debt that silently becomes permanent
- Silent failure handling and silent security degradation
- Undocumented complexity and unreadable “clever” code
- Non-reproducible builds, non-deterministic behavior, and flaky verification
- Work products that are narrative-only, vague, or unverifiable

If a decision conflicts with this covenant, **the decision is wrong**.

---

### A. The Only Acceptable Meaning of “DONE” (The 8-Pillar Standard)

A change is **NOT DONE** until all eight pillars are true:

1. Implementation  
   - Real code exists (not pseudocode). It compiles, runs, and handles edge cases.  
   - Invalid states are rejected; invariants are enforced.  
   - Failure behavior is explicit: timeouts, retries, backpressure, cancellation, and partial failures are defined.

2. Verification  
   - Automated tests cover normal + failure + adversarial + concurrency/ordering behavior.  
   - Tests model production complexity, not toy inputs.  
   - Where appropriate: property-based tests, fuzzing, chaos/fault injection, and regression tests exist.

3. Documentation  
   - In-repo docs explain: architecture, usage, invariants, data contracts, and “what can go wrong.”  
   - Operational caveats are explicit: limits, failure modes, rollback strategy, and safety constraints.  
   - Every public behavior change updates docs and/or changelog.

4. Evidence  
   - Reproducible proof exists (CI artifacts, logs, traces, coverage reports, benchmarks).  
   - Evidence is not implied; it is attached or linkable to a specific commit/run.  
   - If there is no evidence, the claim is false.

5. Traceability  
   - Each requirement / acceptance criterion maps to:
     - code locations,
     - test locations,
     - documentation locations,
     - evidence artifacts.
   - No orphan requirements. No orphan code. No untested requirements.

6. Operational Readiness  
   - Safe defaults, explicit configuration validation, and clear failure signals exist.  
   - Observability exists: structured logs + metrics + tracing (or an equivalent breadcrumb system).  
   - Rollout/rollback exists where relevant; migrations are reversible or explicitly irreversible.

7. Security & Safety Readiness  
   - Threat assumptions are stated. Privilege boundaries are validated.  
   - Secrets are not logged; sensitive data is redacted.  
   - Fail-closed behavior is defined for safety/security modules (no silent “allow”).

8. Task Completeness  
   - Work is decomposed into a concrete task list (not vague bullets).  
   - Each task includes acceptance criteria, a test hook, and an evidence hook.  
   - If a task is not done, the work is not done.

Rule: Declaring “done” without satisfying every pillar is deception.

---

### B. The Anti-Half-Ass Rules (Merge-Blocking by Definition)

#### B1. No Pseudocode-as-Deliverable
- Pseudocode may exist only as clearly labeled design notes.  
- Pseudocode cannot be the “solution,” cannot substitute for tests, and cannot be used to claim completion.

#### B2. No Placeholders / No Stubs / No “Later”
Forbidden in production paths:
- `TODO`, `FIXME`, `pass`, `todo!()`, empty handlers, commented-out behavior, “mock later,” “hardening later,” “edge cases later.”
If incomplete behavior must exist temporarily, it must:
- be feature-flagged OFF by default,
- be isolated so it cannot affect production behavior,
- have a `TD-###` record with TTL (see Debt Policy).

#### B3. No Fake Data, No Toy Inputs, No Lazy Synthetics
- Tests and examples must use production-like complexity: realistic IDs, nested payloads, boundary sizes, malformed variants, weird unicode/whitespace, and adversarial inputs.
- Ban list (unless the test is explicitly about these literals): `foo`, `bar`, `user_1`, `test123`, “hello world.”

#### B4. No Happy Path Testing
For any externally coupled feature (filesystem, subprocesses, network, storage, package backends), tests must cover:
- timeouts, retries, partial failures, malformed responses, permission failures, cancellation, overload/backpressure, and out-of-order/duplicate events where relevant.

#### B5. No Silent Failures
- Swallowing errors is forbidden. Every error must be handled, logged with context, or propagated.
- “Fallback to success” behavior without explicit documentation and tests is forbidden.

#### B6. Warnings Are Errors
- Compiler, linter, formatter, typechecker warnings block merge. “It builds on my machine” is irrelevant.

#### B7. No Unbounded Risk
- Unbounded queues, unbounded memory growth, unbounded metric cardinality, unbounded retries, and unbounded timeouts are forbidden.
- If something can grow, it must have a cap. If it can retry, it must have a budget. If it can wait, it must have a timeout.

---

### C. Technical Debt Policy (Zero Debt Unless Named + Owned + Expiring)

Technical debt is **forbidden by default**. If debt must exist, it must be explicit, bounded, and temporary.

Debt is valid only if it is recorded as `TD-###` and includes:
- owner,
- scope and blast radius,
- why it exists,
- the exact exit criteria (“debt is removed when…”),
- remediation plan,
- hard TTL date,
- tests guarding the boundary so the debt cannot silently expand.

Rules:
- Debt without TTL is invalid.
- Debt past TTL blocks merge/release.
- “We’ll fix later” is not a plan.
- “Temporary” code paths must have an explicit sunset mechanism.

---

### D. SDD + TDD Contract (Professional Standard)

#### D1. Specification-Driven Design Requirements (for non-trivial changes)
A valid spec includes:
- intent and non-goals,
- acceptance criteria (falsifiable),
- invariants (must-always-be-true),
- failure modes and expected behavior under each,
- compatibility rules (protocol/API/schema expectations),
- performance envelope and resource bounds (when relevant),
- security/privacy assumptions and constraints,
- operational concerns (observability, rollout/rollback, migration notes).

If the spec is ambiguous, the first task is to remove ambiguity by producing falsifiable criteria.

#### D2. Test-Driven Development Requirements
- Tests define behavior before/with implementation (TDD by default).
- Refactors require existing tests protecting behavior.
- Every bug fix includes a regression test that fails pre-fix and passes post-fix.
- Flaky tests are critical bugs; they must be fixed, not ignored.

---

### E. Verification Constitution (Realism + Adversarial + Failure-First)

Required test categories (as applicable):
- unit tests for pure logic (fast, no external deps),
- integration tests for boundaries and real dependency interactions,
- property-based tests for parsers/validators/protocol/config boundaries,
- fuzz tests for user-controlled input surfaces,
- concurrency/ordering tests for races, duplicates, replays, idempotency,
- chaos/fault injection for externally coupled behaviors,
- performance regression checks for hot paths or stated latency budgets.

Verification must explicitly test:
- malformed inputs,
- partial reads/writes,
- timeout handling,
- retry policy and idempotency guarantees,
- permission boundaries,
- overload/backpressure behavior,
- deterministic ordering assumptions (or explicit non-guarantees).

---

### F. The Claim Ledger (Mandatory Honesty)

Any claim like “works,” “done,” “fixed,” “secure,” “fast,” “compatible,” “production-ready” must be labeled:

- Observed: executed + evidence attached
- Derived: reasoned + assumptions listed + risks stated
- Unverified: not tested + the exact minimal experiment provided

Presenting Derived/Unverified claims as Observed is lying.

---

### G. Minimum Acceptable Deliverable (Non-Negotiable Output Shape)

Any non-trivial work product must include all of:
- a concrete task list with acceptance criteria per task,
- a file-level plan (what files change/add/remove),
- implementation code,
- tests (including failure/adversarial coverage where relevant),
- documentation updates,
- an Evidence Pack (defined in the footer).

If any part is incomplete, it must be explicitly labeled Unverified and paired with the shortest experiment that would verify it.

### H. Default Principles

If a situation, decision, or design choice is not explicitly covered by this Constitution, the default principle is to:
- **Adopt the most stringent, resilient, and transparent posture.**
- **Enforce the primacy of operational integrity, unambiguous intent, and disciplined scaling.**
- **Treat unresolved ambiguity as a Constitutional void, demanding immediate and formal amendment.**

# I. Vision and Scope

## 1. The Vision

**ArqonHPO is a probe-gated optimization engine for time-to-target.**

It is built to be clearly competitive for two product-aligned use cases:

1. **Fast simulation tuning:** expensive evaluations (milliseconds to seconds) where reaching a useful threshold quickly matters.
2. **Sklearn-style model tuning:** moderate-cost evaluations where optimizer overhead is material and “good-enough quickly” often wins.

## 2. The Scope

To achieve this vision, we must be ruthless about what ArqonHPO **is** and what it **is not**.

### 2.1 In Scope (The Core Product)

ArqonHPO is **probe-gated optimization**. It is responsible for:

* **The Probe Phase:** Deterministic sampling to gather an initial signal and candidates.
* **The Classification Phase:** A fixed-size test that labels the landscape (e.g., structured vs chaotic) and produces a score.
* **The Mode Selection Phase:** Selecting a refinement strategy based on the classification result.
* **The Refinement Phase:** Executing the chosen optimizer within the remaining budget.
* **Audit Artifacts:** Schema-versioned run artifacts sufficient for replay and accountability.

### 2.2 Out of Scope (The Boundaries)

ArqonHPO is **not**:

* **A general-purpose ML training framework.** It tunes; it does not train end-to-end pipelines.
* **A distributed execution platform.** It may integrate with external evaluators, but it does not provide a cluster runtime.
* **A “guaranteed best on all objectives” optimizer.** Claims must be scoped to the benchmark suite and use cases.

## 3. The Strategic Horizon

We define evolution in three distinct epochs. Engineering decisions must align with the current epoch while reserving capacity for the next.

* **Epoch 1: The Foundation (Deterministic probe-gated core).**
    * *Focus:* determinism, bounded overhead, artifact auditability, and time-to-target benchmarking.
    * *Goal:* be measurably competitive on the two target use cases.
* **Epoch 2: The Platform (Composable strategies).**
    * *Focus:* pluggable backends, richer classification signals, replay tooling.
    * *Goal:* support multiple strategies without breaking contracts.
* **Epoch 3: The Research Frontier (Optional).**
    * *Focus:* experimental samplers, meta-controllers, and novel structural probes.
    * *Goal:* enable experimentation without contaminating production defaults.

---

# II. Core Principles

This section defines the engineering laws that govern ArqonHPO. These are not guidelines; they are constraints. Code that violates these principles will be rejected during Review.

### 1. Architectural Invariance (The Gate Pattern)

The system is composed of four non-negotiable phases. **Strict adherence** to the **probe-gated pipeline** is required to prevent coupling and ensure reproducibility.

**The Phases:**

* **Probe:** deterministic sampling; gathers initial candidates and signal.
* **Classify:** fixed-size classification; emits score + label.
* **Select:** chooses refinement mode based on classification.
* **Refine:** executes the chosen optimizer within the remaining budget.

**The Bypass Ban:**

No phase may be skipped because it is “convenient.”

* **Forbidden:** Selecting a refinement mode without running classification.
* **Forbidden:** Adding hidden objective calls that do not count against budget.
* **Forbidden:** Silent fallbacks that change mode/behavior without artifacts and tests.

### 2. Statelessness & State Explicit-ness

To ensure runs can be reproduced and audited, we adhere to a **Stateless Where Possible** philosophy.

* **Run Ephemerality:** Any process must be able to crash and restart without corrupting an ongoing run artifact.
* **State Explicitness:** All non-trivial solver state must be explicit, serializable, and captured in artifacts if it affects decisions.
* **Seed Sovereignty:** All randomness must come from explicit seeds; hidden global RNG use is forbidden.

### 3. Contract Sovereignty (Typed Inputs, Versioned Artifacts)

We enforce a strict separation between machine-stable contracts and human-readable debugging.

**Typed Contracts:**

Configuration and run inputs must be expressed in typed structures. The config contract is the single source of truth for defaults and validation.

**Artifact Reservations:**

Artifacts are schema-versioned and must be stable and replayable. Logs are diagnostics and must not be required for replay.

### 4. Future-Proofing Hooks (The Moonshot Mandate)

To enable future experimentation without requiring a rewrite, v1.0 must reserve capacity for:

* **Backend Hook:** Ability to select a refinement backend via a stable interface.
* **Classifier Hook:** Ability to extend classification signals while preserving the fixed-size gating contract.
* **Objective Guard Hook:** Ability to wrap objectives for timeouts/redaction without changing the solver core.
* **Replay Hook:** Ability to replay decisions from artifacts.

### 5. Semantic Versioning & Compatibility

We adhere to strict Semantic Versioning regarding the **Public API** and **Artifact Schemas**.

**Versioning Rules:**

* **MAJOR:** Breaking changes to solver public API, artifact schema, or core behavior.
* **MINOR:** New modes, additive fields (optional), new telemetry, or non-breaking enhancements.
* **PATCH:** Bug fixes, performance improvements, and clarifications.

**Stealth Ban:**

There shall be no "stealth" breaking changes in MINOR or PATCH versions. Ever.

### 6. Data Isolation & Privacy (The Bulkhead)

Optimization objectives may embed sensitive information.

* **Isolation:** Objective payloads are treated as sensitive by default.
* **Redaction:** Logs/artifacts must not leak secrets or tenant data.
* **Sharing Safety:** Artifacts intended for sharing must support redaction without breaking replayability guarantees.

### 7. Security by Design

Security is a baseline constraint, not a feature.

* **Zero Trust:** The solver does not trust the objective; it validates inputs and guards execution.
* **Fail Closed:** If a safety/guard module fails, times out, or crashes, the run is failed explicitly. It is never "allowed by default."
* **Secure Defaults:** Unsafe execution modes must be opt-in and auditable.

### 8. Programmable Safety (Guards)

We reject hardcoded, silent safety logic. Safety requirements vary by environment.

* **Guard Middleware:** Objective guards (timeouts, resource caps, redaction) must be composable.
* **Fail Closed Mandate:** If a guard times out, crashes, or returns an error, execution is blocked and surfaced.
* **Bounded Execution:** All guards must have strict, non-negotiable limits.

### 9. Run-as-Artifact (The Capsule Principle)

Runs are not just ephemeral computations. They must produce replayable artifacts.

* **Digital DNA:** The system is optimized to transport the *potential* (seed + config + bounds + decisions), not just the result.
* **Replayability:** A run can be reconstructed from the artifact plus the objective.

### 10. Delta-First Evidence

Computation cost and audit burden scale with **change ($\Delta$)**.

* **Diff over Snapshot:** Prefer per-eval traces and incremental evidence over opaque summaries.
* **Causal Integrity:** Decisions and their inputs must be recorded in order.

### 11. Circuit-First Benchmarking

Benchmarks are declarative.

* **Bench Suites are Circuits:** Benchmark suites define objectives, budgets, targets, and seed suites as configuration.
* **Decoupled Objectives:** Objectives remain oblivious to optimizer internals.

---

### Tier Ω Scope (Applies to Sections 12–21)

Sections **12–21** govern **Tier Ω (Experimental)** features only.

Tier 1 (Production) behavior **must** continue to satisfy all Core Principles in:

* **Section II.1–11** (Pipeline integrity, determinism, contracts, privacy, boundedness),
* **Section VIII** (Performance & Hot-Path Invariants),
* **Section IX** (Observability & Telemetry Contracts), and
* **Section X** (Data Governance & Retention),

regardless of any Tier Ω configuration. **No Tier Ω behavior may weaken or bypass those invariants.**

---

### 12. Bounded Emergence (Tier Ω Only)

We consciously work only inside the **Engineerable Sub-Space**.

* **Tier Ω Only:** Even in experimental regimes, we serve systems where we retain control (budget, timeouts, caps).
* **Chaos Ban (Default):** Highly chaotic or poorly characterized regimes are considered *out of scope* for production claims and live only in research sandboxes.
* **Core Invariant Link:** All Tier Ω work remains subject to Determinism (II.2), Privacy (II.6), Safety (II.7–8), and Boundedness (VIII.1–2).

### 13. Temporal Sovereignty (Tier Ω Only)

**Time-varying structure** (adaptive schedules, time-to-target controllers) is a first-class control mechanism.

* **Dynamic Schedules:** We assume control can be restored through well-designed temporal programs, not just static settings.

### 14. Mathematical Rigor (Algebraic Preference, Tier Ω Only)

* **Solvers over Heuristics:** If a problem can be solved by a matrix operation or algebraic solver, do not use a neural net or heuristic.
* **Explicit Control:** Controllers must be explicit and observable. Hidden control loops are forbidden.
* **Structured Sampling:** Prefer deterministic low-discrepancy / structured grids over naive random sweeps for discovery loops.

### 15. The Omega Tier (Risk Classification, Tier Ω Only)

Strategies are classified by risk profile:

* **Tier 1 (Production):** Safe, bounded, and deterministic for benchmark claims.
* **Tier Ω (Experimental):** Permitted to explore complex behavior but must be strictly confined and never become default by accident.

### 16. Diagnostic Segregation (Tier Ω Only)

* **Signals vs Decisions:** Outputs from Ω-tier probes are treated as **Diagnostic Signals**, not direct decision-makers in production defaults.

### 17. The 4-Layer Hierarchy (Tier Ω Only)

Complex systems follow the standard **Substrate → Observer → Controller → Architect** hierarchy.

* **Explicit Roles:** Components must implicitly or explicitly fulfill one of these roles.
* **Recursive Operators:** Recursive strategies must explicitly declare recursion depth limits and halt conditions.
* **Meta-Optimizers:** Meta-optimizers whose output is the configuration of other optimizers are subject to strict evidence and safety gates.

### 18. Probability Engines (Tier Ω Only)

The system may support probability-shaping engines where outputs are distributions, not scalars.

* **Superposition:** Decisions may be probabilistic until explicitly collapsed by a deterministic selection rule.

### 19. Temporal Physics (Tier Ω Only)

* **Phased Operation:** Systems may explicitly declare phases (e.g., `[
  "probe",
  "classify",
  "refine"
]`). Control policies must adapt to the active phase.

### 20. The Reality Factory (Tier Ω Only)

The system may manage governed experiment namespaces (“Realities”) as first-class lifecycle objects.

* **Lifecycle States:** Experiments must track lifecycle state (`Draft` → `Running` → `Promoted` → `Archived`) with explicit transition gates.

### 21. Strong Emergence Patterns (Tier Ω Only)

* **Homeostatic Override:** Controllers must have the authority to force reset when error thresholds are breached.
* **Curiosity Metrics:** “Surprise” is a valid optimization signal for discovery-only operators.

---

# III. Code Quality & Engineering Standards

### 1. The "Boring Code" Manifesto

We value clarity over cleverness. ArqonHPO must be readable by a junior engineer at 3 AM.

* **Readability First:** If a "clever" one-liner creates cognitive load, expand it.
* **Explicit over Implicit:** Magic behavior, monkey-patching, and hidden control flow are forbidden.
* **Standard Tooling:** We adhere strictly to community standards (formatters, linters, type checkers).

### 2. Asynchronous Boundaries

If concurrency/parallel evaluation exists, it must not destroy determinism.

* **Purity Mandate:** Core decision logic must remain synchronous and pure where possible.
* **Timeout Mandate:** No external call (objective, subprocess, IO) shall exist without a configured timeout.

### 3. Error Handling Philosophy

Errors are data, not exceptions. They must be handled explicitly.

* **Fail Loud (Developer Errors):** Logic errors and invariant violations must crash or hard-fail immediately.
* **Fail Soft (Runtime Errors):** External failures must be handled via explicit rejection or controlled degradation.
* **The "Swallow" Ban:** Silent discard of errors is forbidden.

### 4. Logging & Observability

* **Structured Only:** Logs must be structured and include correlation identifiers (e.g., `run_id`).
* **Level Discipline:** `ERROR` means operator intervention is required. `WARN` means handled anomaly. `INFO` is lifecycle.
* **Security Redaction:** Logs must never contain sensitive objective payloads at `INFO` or above.

### 5. Configuration Discipline

* **Config Over Code:** Operational thresholds (timeouts, budgets, caps) must be configurable, not magic numbers.
* **Validation on Startup:** Invalid configuration must fail fast with explicit errors.

### 6. Deterministic State & Contract Correctness

Optimization systems die when state becomes ambiguous.

* **State Machine Contracts:** Phase transitions and mode decisions must be explicit.
* **Determinism:** Same seed + same env + same objective → same decisions and results (within defined tolerances).
* **Artifact Contracts:** Artifact schema is a compatibility surface.

### 7. Contract-First Definition

* Typed config and artifact schemas are the source of truth.
* Untyped ad-hoc dictionaries/maps in core paths are prohibited.

### 8. Memory Safety & Resource Guarantees

* **Resource Caps:** Every subsystem must define caps for memory, retries, and timeouts.

### 9. Concurrency Safety & Ordering

* **Ordering Invariants:** The solver must never assume ordering unless it enforces it.

### 10. Performance Discipline

* **Hot Path Hygiene:** Avoid unnecessary allocations and logging inside per-eval loops.
* **Latency Budgets:** If a latency budget exists (time-to-target), it must be measured and guarded.

### 11. API & Interface Stability

* **Boundary Contracts:** Internal modules communicate via stable interfaces.

### 12. Dependency Hygiene

* **Admission Rules:** New dependencies are guilty until proven innocent.
* **Pinning:** Benchmark-critical dependencies must be version-pinned.

### 13. Documentation Standards

* **Docs as Code:** Documentation must live in the repo.
* **Decision Records:** Significant decisions must be captured (ADR or equivalent).

### 14. Build & Artifact Integrity

* **Reproducibility:** Builds and benchmark runs must be reproducible.
* **Binary/Package Hygiene:** Produced artifacts must be traceable to a commit and environment.

### 15. Mathematical Rigor (Algebraic Preference)

* **Solvers over Heuristics:** Prefer explicit solvers where applicable.
* **Explicit Control:** Hidden control loops are forbidden.
* **Structured Sampling:** Prefer deterministic sampling schemes where it improves time-to-target and reproducibility.

---

# IV. Testing Strategy & Quality Gates

### 1. TDD as the Working Standard

Test-Driven Development (TDD) is the **default and expected workflow** for all ArqonHPO components.

* **The Workflow:**
  1. **Specify:** Define behavior in `/specs/` (SDD-first).
  2. **Test:** Write or extend tests that express that behavior.
  3. **Implement:** Write the code that satisfies the tests.
  4. **Refactor:** Optimize while keeping the suite green.

### 2. Coverage Expectations (Per Subsystem)

Coverage is about behavioral exhaustiveness, not raw percentages.

* **Solver Core:** Must cover probe, classify, mode select, refine, and budget accounting.
* **Artifact Layer:** Must cover schema versioning, determinism, replay-critical fields.
* **Benchmark Harness:** Must cover time-to-target measurement and reporting.

### 3. Test Discipline Requirements

* **Unit Tests:** Must run fast with zero external services.
* **Integration Tests:** Must run end-to-end with real dependencies where relevant.
* **Flaky Tests:** Flaky tests are **Critical Bugs**.
* **Determinism:** Tests must avoid random sleeps and time-dependent logic; use controlled clocks.

### 4. Quality Gates

A PR **may not be merged** if any of the following are true:

* **Determinism Gate:** nondeterministic behavior without explicit labeling and tests.
* **Evidence Gate:** benchmark/perf claims without reproducible evidence.
* **Artifact Gate:** schema changes without versioning and compatibility notes.
* **Spec Gate:** behavior implemented without a Spec, or Spec not updated to match Code.
* **Technical Debt Gate:** new `TODO`s without `TD-###` and TTL.

---

# V. Lifecycle & Automation

ArqonHPO does not “ship code”; it manufactures **artifacts** through a controlled factory.

### 1. The Factory Mandate

Manual releases are forbidden for production claims. CI is the source of truth.

* **The Pipeline is Sovereign:** If it did not pass CI, it does not exist.

### 2. Immutable & Reproducible Artifacts

* **Immutable Artifacts:** Release artifacts must be identifiable by content hash.
* **Reproducible Builds:** The same commit must build reproducibly in the canonical environment.

### 3. Supply Chain Security

* **Dependency Locking:** No floating versions for benchmark-critical paths.
* **Provenance:** Artifact origin must be traceable (commit, branch, CI run).

---

# VI. Operational Excellence

ArqonHPO is correctness-sensitive infrastructure for optimization. The way it behaves under real objectives is as important as the way it behaves in tests.

### 1. Performance & Capacity Invariants

* **Boundedness:** No unbounded loops. Budget and timeouts are mandatory.
* **Overhead Discipline:** Policy updates and bookkeeping must remain bounded and low overhead.

### 2. Observability & Audit

* **Reconstructability:** Logs + artifacts must allow reconstruction of what happened in a run.
* **No Silent Recovery:** Any fallback must be visible and test-covered.

---

# VII. Governance & Amendment

Governance defines how ArqonHPO protects its mission and how this Constitution itself may change.

### 1. Scope Protection

ArqonHPO is probe-gated optimization for the two target use cases. Scope creep is a bug.

### 2. Complexity Budget

Complexity is technical debt with compound interest.

* Adding major dependencies or new execution modes requires explicit review and an ADR.

### 3. Amendments

This Constitution is living but intentionally hard to change.

* Amendments require a documented proposal (rationale + impact) and a version bump.

---

# VIII. Performance & Hot-Path Invariants

Performance is not an optimization; it is a **correctness property**.

### 1. Boundedness as Law

Unbounded anything is a denial-of-service vector.

* **No Unbounded Work:** Every loop must have a budget.
* **CPU Boundaries:** Heavy work must not block the per-eval control loop without explicit design.

### 2. Hot-Path Constraints

* **O(1) or Amortized O(1):** Per-eval policy decisions must be O(1) or amortized O(1).
* **No Hidden I/O:** Do not write artifacts inside the inner loop unless explicitly buffered.

---

# IX. Observability & Telemetry Contracts

What cannot be observed cannot be governed.

### 1. Logs, Metrics, Traces as First-Class Citizens

* **Structured Logs Only:** Must include `run_id` and phase markers.
* **Telemetry for Mode Decisions:** Mode selection and classification results must be observable.

---

# X. Data Governance & Retention

ArqonHPO may handle sensitive objective data. Data is an asset and a liability.

### 1. Data Classification

* **Run Artifacts:** schema-versioned run outputs.
* **Objective Data:** treated as sensitive by default.

### 2. Retention

* **Explicit Retention:** No infinite retention by accident; retention policies must be explicit.

---

# XI. Internal Service Contracts & Complexity Escalation

The internal structure must remain understandable, evolvable, and safe.

### 1. Internal Contracts

* **Versioned Contracts:** Internal boundaries must have explicit, versioned contracts (types + schemas).

### 2. Complexity Budget & Escalation

* Introducing a new core dependency or major execution mode requires a design review document and ADR.

---

# XII. Glossary & Canonical Definitions

To prevent interpretation drift (especially for Spec Kit agents), we define core vocabulary used throughout this Constitution.

| Term | Definition |
| :--- | :--- |
| **ArqonHPO** | The probe-gated optimization engine described by this Constitution. |
| **Probe** | Deterministic initial sampling phase to gather candidates and signal. |
| **Classify** | Fixed-size test producing a label and score to drive mode selection. |
| **Mode** | The chosen refinement strategy family (structured vs chaotic). |
| **Time-to-Target** | Time/evals to reach a specified objective threshold. |
| **Tier 1** | Production behavior and benchmark claims subject to all core constraints. |
| **Tier Ω** | Experimental features allowed only under strict confinement; never default. |

---

# XIV. ULTIMATE INTEGRITY ATTESTATION & EVIDENCE PACK

This section is the **merge/ship gate**. It exists so “done” is not a feeling—it is a reproducible fact.

---

### 1) Merge/Ship Attestation (Required)

By merging or shipping, the author(s) and reviewer(s) attest:

- No placeholders exist in production paths (no TODOs, stubs, pseudocode-as-work, “later hardening”).
- No fake evidence is presented (no invented logs, benchmarks, screenshots, coverage, or results).
- No happy-path-only verification exists for critical behaviors.
- No silent failure handling exists; errors are handled/logged/propagated with context.
- Warnings were treated as errors (clean lint/typecheck/compile).
- Any technical debt is recorded as `TD-###` with owner + TTL + exit criteria and is bounded by tests.
- All claims are labeled Observed/Derived/Unverified, and Observed claims have attached evidence.

If you cannot honestly attest to every item above, you must not merge/ship.

---

### 2) Evidence Pack (Attach or Link; Required)

A change is invalid without a reproducible Evidence Pack. The Evidence Pack must be tied to a specific commit and must be reproducible by another engineer.

#### 2.1 Build Proof
- CI run or local output showing:
  - clean build,
  - clean lint/typecheck/format,
  - warnings treated as errors.

#### 2.2 Test Proof
- Results for:
  - unit tests,
  - integration tests (where applicable),
  - property/fuzz tests (where required by input boundaries),
  - concurrency/ordering tests (where applicable).
- A short note listing what is not covered and why (explicitly, not implicitly).

#### 2.3 Failure Matrix Proof (Where the bad paths live)
For each externally coupled feature, list:
- failure scenarios tested (timeouts, retries, malformed responses, permission failures, overload/backpressure, partial failures),
- test file(s) and test names (or equivalent pointers).

#### 2.4 Traceability Proof (Truth Table)
Provide a “truth table” mapping:
- requirement / acceptance criteria → implementation location(s) → test location(s) → documentation location(s) → evidence artifact(s).

Rule: If a requirement has no test, it is untested. If a test has no requirement, it is suspicious.

#### 2.5 Runtime Proof (When Applicable)
- example run logs demonstrating:
  - normal behavior,
  - at least one failure mode behaving correctly.
- proof of observability works:
  - correlation IDs exist,
  - metrics/traces exist (or equivalent breadcrumbs).

#### 2.6 Performance / Resource Proof (When Relevant)
- baseline numbers + method + environment,
- a regression guard (benchmark test, threshold check, or documented budget),
- proof of bounded behavior (caps, backpressure, shedding policy).

#### 2.7 Reproduction Commands
- one-command verification (examples):
  - `pytest`, `python -m build`, etc.
- environment notes:
  - pinned toolchains/dependencies,
  - seed control for deterministic tests.

---

### 3) Debt Register Enforcement (TD-###)

If any `TD-###` exists in the change:
- TTL date and owner are mandatory.
- The debt boundary must be protected by tests so it cannot silently expand.
- The exit criteria must be concrete.
- Debt past TTL is a release/merge blocker.

---

### 4) Professional Review Checklist (Hard Questions Only)

Review must answer “yes” with evidence:

- Does this handle failure modes explicitly (not “assumed”)?
- Are tests realistic, adversarial, and non-trivial (no lazy synthetics)?
- Are there concurrency/ordering hazards, and are they tested or explicitly ruled out?
- Is behavior observable (logs/metrics/traces/breadcrumbs)?
- Are resource bounds explicit (timeouts, caps, retry budgets, queue bounds)?
- Is the code readable under pressure (3 AM standard)?
- Is documentation updated to match behavior and constraints?
- Can another engineer reproduce the Evidence Pack from scratch?

If any answer is “no,” the change is not complete.

---

### 5) Claim Ledger Summary (Required When Stating Status)

If a deliverable claims completion or correctness, it must include:

- Observed claims: link evidence
- Derived claims: list assumptions + risks + how to verify
- Unverified claims: list the minimal experiment to verify

Rule: If it cannot be reproduced from the Evidence Pack, it is not true.  
Rule: If it is not true, it is not done.

---

## Implementation Substrate & SDK Contract
- Core implementation MUST be a Rust library crate exposing the probe-gated solver API.
- CLI MUST be a thin Rust binary crate that delegates to the core.
- SDKs (for example, Python) MUST be thin bindings over the same core, not reimplementing solver logic.
- Artifacts MUST be language-agnostic (JSON) and serve as the compatibility contract between surfaces.

**Version**: 1.0.0  
**Ratified**: 2025-12-13  
**Last Amended**: 2025-12-13  
