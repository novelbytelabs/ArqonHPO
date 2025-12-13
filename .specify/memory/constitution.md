<!--
Sync Impact Report:
- Version: 2.4.0 -> 2.4.1
- Modified principles:
  - I. Warm-Start Pipeline Integrity (expanded phase definitions, override controls, rolling mode switch rules)
  - II. Reproducibility & Statelessness (made environment capture and determinism requirements explicit)
  - III. Interface & Protocol Discipline (added artifact compatibility + hot-path boundary rules)
  - IV. Test-First Quality Gates (expanded required test types and determinism rules)
  - V. Observability, Performance & Safety (expanded boundedness + timeout + failure-mode logging)
  - VI. Canonical Environment & Dependency Determinism (clarified: canonical env is for repo validation/benchmarks; end users are not forced to use conda or a specific env name)
- Added principles:
  - VI. Canonical Environment & Dependency Determinism (conda env `helios-gpu-118`)
  - VII. Artifact Contracts & Auditability
  - VIII. Research / Production Boundary
  - IX. Data & Privacy Safety
  - X. Compatibility & Versioning Discipline
- Added sections:
  - Expanded Engineering Standards & Constraints (determinism, numerical hygiene, environment)
- Removed sections: None
- Templates requiring updates:
  - ✅ .specify/templates/plan-template.md
  - ✅ .specify/templates/spec-template.md
  - ✅ .specify/templates/tasks-template.md
  - ✅ .specify/scripts/bash/update-agent-context.sh (default agent file now AGENTS.md)
  - ✅ README.md (document canonical conda env)
  - ✅ environment.yml (new; documents canonical env)
  - ⚠ N/A: .specify/templates/commands/*.md (directory not present)
- Follow-up TODOs: None
-->

# ArqonHPO Constitution

**Version**: 2.4.1  
**Ratification Date**: 2025-12-11  
**Last Amended**: 2025-12-12

This document codifies the non-negotiable rules for the ArqonHPO warm-start
optimization engine. If a decision conflicts with this constitution, the
decision is wrong.

This constitution exists to prevent:
- warm-start bypasses that silently destroy quality and reproducibility,
- uncontrolled heuristics in the hot path,
- “works on my machine” benchmarking and environment drift,
- nondeterministic results that cannot be audited or rerun.

Specs, plans, and tasks generated via speckit must comply.

**Scope (what this governs)**:
- The warm-start optimization pipeline (RPZL probe, variance test, mode selection, refinement).
- The public API surface (`ArqonSolver`) and any CLI wrappers.
- Artifacts produced for audit (samples, rationale, metrics, environment).
- Benchmark claims and published performance numbers.

**Non-scope (what this does not govern)**:
- The internal architecture of ArqonBus (covered by ArqonBus governance).
- One-off research notebooks and prototypes that do not ship as supported surfaces
  (still must not be misrepresented as production results).

## Core Principles

### I. Warm-Start Pipeline Integrity
Rationale: warm-start is the product; bypasses destroy correctness and trust.

Non-negotiables:
- Solver flow is always: RPZL (prime-indexed) probe → variance classification → seeded refinement.
- Landscape classification is required: run the RPZL variance test (50 RPZL samples,
  neighbor variance) before mode selection.
- Default solver mode is Hybrid RPZL+Optuna; pure Optuna is allowed only when variance
  indicates chaotic/rugged landscapes (e.g., Rastrigin-class).
- No “helpful” heuristics may be inserted into the hot path unless:
  - they are spec’d, tested, and benchmarked, and
  - they do not bypass the RPZL probe + variance decision.

Overrides and bypass controls:
- Any override/bypass (including “force pure Optuna”) MUST be explicit, auditable, and time-bounded.
- Overrides MUST include a rationale string and an external reference (ADR link or issue ID).
- Overrides MUST be off by default in production surfaces.

Phase definitions (must be observable):
- **Probe**: RPZL sampling used to discover structure and gather probe samples.
- **Classify**: variance test produces a structured/chaotic label and score.
- **Seed**: hybrid mode seeds refinement from probe findings (when applicable).
- **Refine**: Optuna/TPE (or approved Bayesian backend) refines within the selected mode.

Rolling classification (long-running workloads):
- The solver MUST support a rolling window (default: last 50 relevant samples) and MAY switch
  modes when the classification changes.
- Any mode switch MUST be logged with the old/new mode and the variance score(s) that triggered it.

### II. Reproducibility & Statelessness
Rationale: optimization results are only valuable if they can be reproduced and audited.

Non-negotiables:
- Every run MUST be reproducible: fixed seeds, recorded bounds/budget/probe_ratio, and deterministic
  objective evaluation (given the same environment and objective).
- No hidden global state or cached mutations; all state is derived from inputs or explicit artifacts.
- The solver MUST be restartable from serialized artifacts without changing behavior.

Required run metadata (minimum):
- All RNG seeds used (probe sampling, variance test, refinement backend).
- Bounds, budget, probe_ratio, and any constraint definitions.
- RPZL parameters (prime set/stride choice, sample count, neighborhood definition).
- Variance test result (score, threshold(s), label).
- Selected mode and (if hybrid) the seed mapping used to initialize refinement.
- Dependency versions and environment fingerprint (see Principle VI).

Determinism guardrails:
- Randomness MUST come from an explicit RNG seeded from the run inputs.
- If parallelism is used, the run MUST still be deterministic or MUST declare nondeterminism explicitly
  and treat the output as non-auditable (not allowed for benchmark claims).

### III. Interface & Protocol Discipline
Rationale: stable, typed contracts keep the solver composable and safe to embed.

Non-negotiables:
- Primary product surface is a stable API (`ArqonSolver` semantics) exposed in multiple runtimes.
- Inputs/outputs use typed, explicit structures; logs/JSON are diagnostics only and stay out of the
  optimization loop.
- Public contracts are versioned: changes to solver signatures or return types follow SemVer with
  additive-first changes and documented deprecations.

Artifact and logging boundaries:
- Diagnostics may be verbose; the optimization loop must remain free of non-essential I/O.
- Artifacts MUST be structured, schema-versioned, and backward compatible (see Principle X).

### IV. Test-First Quality Gates
Rationale: solver logic is easy to regress; tests are the only reliable guardrail.

Non-negotiables:
- TDD is mandatory for solver logic, sampling math, and integrations (Optuna, NumPy, SciPy).
- Regressions block merges.
- Flakiness or nondeterminism is treated as a defect to fix, not a tolerance to accept.

Required coverage (by behavior, not percentage):
- Unit tests for: RPZL sampling invariants, variance test computation, and mode selection policy.
- Integration tests for: end-to-end warm-start flow in both structured and chaotic fixtures.
- Property tests for: sampling distribution sanity (where applicable).
- Determinism tests: same seed + same env yields same mode and identical (or within defined epsilon)
  outputs for deterministic objectives.

### V. Observability, Performance & Safety
Rationale: predictable performance and clear diagnostics are correctness properties.

Non-negotiables:
- Structured logging around phases (probe, classify, seed, refine) with elapsed times, seed metadata,
  and solver selection rationale (variance result).
- No sensitive data in logs (objective payloads are treated as potentially sensitive).
- No unbounded loops: every loop MUST have a budget or termination criterion tied to configured inputs.
- Objective wrappers MUST guard against runaway compute and memory, including timeouts.

Required telemetry fields (minimum):
- `run_id`, `seed`, `bounds_digest`, `budget`, `probe_ratio`
- `rpzl_sample_count`, `rpzl_prime_stride` (or equivalent RPZL parameterization)
- `variance_score`, `variance_label`, `mode_selected`
- phase timings: `probe_ms`, `classify_ms`, `refine_ms`, `total_ms`

Spectral telemetry:
- When enabled, the solver SHOULD emit a “structural clarity” signal (e.g., eigen-gap or related metric)
  and record it in artifacts for post-hoc analysis.

Hot-path boundaries:
- For microsecond-class workloads and benchmark claims, hot-path search logic MUST run in Rust with
  pre-allocated buffers and no GC pauses; Python bindings must remain outside the control loop.

Safety:
- Any failure in classification, seeding, or refinement MUST surface as an explicit error state with
  rationale, not a silent fallback to an arbitrary mode.

### VI. Canonical Environment & Dependency Determinism
Rationale: “same seed” is meaningless if the environment drifts.

Non-negotiables:
- The canonical development and benchmarking environment is a conda environment defined by
  `environment.yml` (internally referred to as `helios-gpu-118`).
- Benchmark claims MUST be produced using the canonical dependency set from `environment.yml`
  (or a documented, equivalently pinned and reviewed alternative).
- CI MUST create its own environment from `environment.yml` and is allowed to name it generically
  (e.g., `ci`); the environment name is not a customer-facing contract.
- Installing Python packages outside the active canonical conda environment is forbidden for this
  project.
  Specifically, `pip install --user` and writing to `~/.local/lib/python*` is forbidden.
- Automation MUST prevent user-site leakage by setting `PYTHONNOUSERSITE=1` for runs that must be
  reproducible (tests, benchmarks, CI).
- Package installs MUST be explicit and env-scoped (e.g., `python -m pip install --no-user ...`
  executed from within the `helios-gpu-118` interpreter).

End-user compatibility (non-negotiable):
- The published Python package MUST NOT require conda, MUST NOT require a specific conda env name,
  and MUST remain usable in standard Python environments via `pip` (subject to documented Python
  version support).

Required environment capture for auditable runs:
- Record environment fingerprint in run artifacts:
  - conda environment name (`helios-gpu-118`),
  - an exported dependency list (`conda env export --name helios-gpu-118` or equivalent),
  - OS + CPU/GPU identity (when relevant to numeric determinism),
  - Python package versions (pip/conda).

### VII. Artifact Contracts & Auditability
Rationale: optimization is only trustworthy if it leaves an audit trail.

Non-negotiables:
- Intermediate artifacts MUST be serializable for audit and rerun.
- Artifacts MUST be stable, schema-versioned, and non-executable by default.
- Pickle-based primary artifacts are forbidden (allowed only for strictly internal debugging).

Minimum artifact set per run:
- Probe samples (or a digest + replayable generator parameters).
- Variance test result and mode selection rationale.
- Final best candidate(s) and objective value(s).
- Run metadata and environment fingerprint.

### VIII. Research / Production Boundary
Rationale: exploration is good; shipping unvalidated novelty is not.

Non-negotiables:
- Experimental kernels, alternate samplers, and novel execution modes must remain confined to
  research artifacts until promoted via: spec → tests → benchmarks → docs update.
- Production library surfaces remain stable; experimental code must not silently change behavior
  of the default solver path.

### IX. Data & Privacy Safety
Rationale: objectives may embed tenant or sensitive information.

Non-negotiables:
- Objective evaluation MUST not leak sensitive data through logs, artifacts, or exceptions.
- Sample logging is opt-in and MUST be redactable.
- Artifacts intended for sharing MUST support redaction without breaking replayability guarantees.

### X. Compatibility & Versioning Discipline
Rationale: reproducibility and downstream integrations depend on stability.

Non-negotiables:
- `ArqonSolver` public API and artifact schemas follow SemVer:
  - MAJOR: breaking API or artifact schema changes,
  - MINOR: additive fields/features with defaults,
  - PATCH: bug fixes and clarifications without contract changes.
- Deprecations MUST be explicit and documented before removal.
- Artifact schema versions MUST be recorded in every artifact bundle.

## Engineering Standards & Constraints
Engineering rules apply to all production-facing code and benchmark claims.

Docs and decision capture:
- Docs as code: README/docs/notebooks must stay in sync with solver behavior and benchmarks.
- ADRs capture significant algorithmic choices (variance thresholding, prime strategies, mode switching).
- `docs/rpzl_adjoint_hybrid.ipynb` is canonical for RPZL structure/chaos guidance and hybrid benchmarks;
  maintain its findings in line with code.

Objective purity and execution boundaries:
- Objective functions MUST be pure and side-effect free (no hidden I/O, no hidden global mutation).
- Stateful setup belongs outside solver inputs and is passed explicitly.
- Any caching must be explicit, keyed, and captured in artifacts if it affects behavior.

Numerical and determinism hygiene:
- Floating-point tolerance MUST be defined where exact equality is not reasonable.
- GPU-dependent behavior MUST be declared; benchmark claims MUST document hardware and driver constraints.
- Nondeterministic kernels are forbidden in benchmark claims unless explicitly justified and isolated.

Environment discipline:
- Default execution and benchmarking uses conda env `helios-gpu-118` (see Principle VI).
- Dependencies are pinned (requirements and environment spec); unbounded version ranges are forbidden
  for benchmark-critical paths.

CI is sovereign:
- CI is the source of truth: linting, type checks, tests, and benchmarks must pass.
- Manual hotfixes or skipped checks are forbidden.

## Development Workflow & Quality Gates
- Standard flow: `/speckit.specify` → `/speckit.clarify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement`; each step must honor this constitution.
- Plans enumerate technical context, constitution gates, and chosen structure; tasks map directly to user stories and include file paths and `P` markers where safe.
- Solver-mode decision (structure vs chaos) from the variance test must be captured in specs/plans and enforced in tasks; deviations need ADR justification.
- Tests run before code for each task phase; integration and benchmark scripts (Ackley/Rosenbrock/Rastrigin, hybrid head-to-heads) must be runnable locally and in CI without hidden prerequisites.
- Releases are tag-driven via CI pipelines; artifacts are immutable, signed where applicable, and never built from unverified local states.
- Documentation and checklists update alongside behavior; no feature is done until spec, plan, tasks, and docs reflect reality.
- Plans MUST explicitly name the execution environment used for benchmarks (default: `helios-gpu-118`).

## Governance
- This constitution supersedes other practices for ArqonHPO; violations block merges and releases.
- Amendments require a documented proposal (rationale + impact), maintainer consensus, a version bump,
  and updated ratified/amended dates.
- Semantic versioning applies to this document:
  - patch = clarifications/typos/non-semantic refinements,
  - minor = new constraints/sections or materially expanded guidance,
  - major = removal/redefinition of principles or governance breaking shifts.
- Compliance review expectation: every plan MUST include a “Constitution Check” section listing the
  applicable gates and how they are satisfied.
- Exceptions require explicit, time-bounded ADRs and MUST not become the default path by accident.
