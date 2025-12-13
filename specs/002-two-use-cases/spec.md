# PRD: ArqonHPO v1 – Probe‑Gated Optimization (Two Use Cases)

**Project**: ArqonHPO  
**Spec ID**: 002-two-use-cases  
**Status**: Draft (v1.0, Rust core baseline)

---

## 1. Product Overview

ArqonHPO v1 is a **probe‑gated optimization engine** focused on two concrete, high‑value use cases:

1. **Fast simulation tuning (time‑to‑target)**  
   Expensive objective evaluations (milliseconds–seconds) on mostly smooth / structured landscapes, where reaching a "good enough" configuration quickly matters more than squeezing out the last few percent.

2. **Sklearn‑style model tuning (time‑to‑target)**  
   Moderate‑cost ML objectives where optimizer overhead is material and "good‑enough quickly" often beats "best eventually."

The goal is **not** to be universally SOTA on every search space; the goal is to be:
- **Clearly competitive and explainable** on these two use cases, and
- **Deterministic, auditable, and bounded‑overhead** by construction.

At its core, ArqonHPO always follows the same pipeline:

> **probe → fixed‑size classify → mode select → refine**

---

## 2. Problem & Goals

### 2.1 Problem

General‑purpose HPO (e.g., pure TPE) often:
- Is **too slow** to reach a useful target when evaluations are expensive.
- Is **overhead‑dominated** when evaluations are moderately cheap.

Current tools also frequently lack:
- Strict determinism given seed and environment.
- First‑class artifact contracts that allow replay and audit.

### 2.2 Goals (v1)

1. **Time‑to‑target improvement (sim tuning)**  
   For structured, expensive objectives, ArqonHPO should reach target thresholds **faster or more reliably** than pure TPE, across a benchmark suite and seed set.

2. **Competitive time‑to‑target (sklearn tuning)**  
   For small/moderate ML objectives, ArqonHPO should deliver **competitive or better** time‑to‑target versus TPE, while keeping optimizer overhead predictable and bounded.

3. **Deterministic, auditable runs**  
   Same objective + bounds + budget + seed + environment ⇒ identical decisions, identical best value (up to numeric tolerance), and replayable artifacts.

4. **Bounded overhead & no hidden work**  
   The policy/selection logic must be O(1) per evaluation, and **must not** introduce extra objective calls beyond the configured budget.

### 2.3 Non‑Goals

- Being SOTA on all objective classes (e.g., highly chaotic, arbitrary black‑box landscapes).
- Distributed/async evaluation in v1 (single‑process focus).
- Full support for arbitrary discrete / categorical‑only spaces in v1.

---

## 3. Users & Use Cases

### 3.1 Personas

1. **Simulation engineer ("Sim Eng")**
   - Owns a simulator or numerical kernel.
   - Evaluations are expensive; cares about **time‑to‑target** and repeatability.

2. **Applied ML engineer ("ML Eng")**
   - Tunes sklearn‑style models (classifiers/regressors).
   - Evaluations are moderate cost; cares about **optimizer overhead** and predictable behavior.

### 3.2 User Stories (Prioritized)

#### US1 (P0): Fast simulation tuning

**As a** Sim Eng,  
**I want** the optimizer to reach a useful objective threshold quickly,  
**so that** I can iterate on simulation configs without waiting for long runs.

**Acceptance:**
- Given bounds, seed, and budget, when I run on an expensive smooth objective, the optimizer reaches the agreed target threshold **faster in median time‑to‑target** than pure TPE across a seed suite.
- Given the same inputs and environment, rerunning produces **identical artifacts and best value** (up to numeric tolerance).

#### US2 (P0): Sklearn‑style ML tuning

**As a** ML Eng,  
**I want** competitive time‑to‑target without large optimizer overhead,  
**so that** model selection is fast and repeatable.

**Acceptance:**
- Given a sklearn objective and fixed seed/budget, the optimizer s median time‑to‑target is **competitive with or better than** TPE at agreed thresholds.
- Given `PYTHONNOUSERSITE=1`, benchmark runs import successfully and results are stable (no user‑site leakage).

---

## 4. Core Behavior & Requirements

### 4.1 Pipeline Definitions

- **Probe**: deterministic, low‑overhead sampler that produces an initial set of candidate evaluations.
- **Classification**: fixed‑size test (default: 50 samples) that labels the landscape as **structured** or **chaotic** and outputs a score.
- **Mode selection**: decision step mapping classification output ⇒ refinement mode.
- **Refinement**: follow‑on optimization within the remaining budget.
- **Time‑to‑target**: wall‑clock time or evaluation count to reach a specified objective threshold.

### 4.2 Non‑Negotiables (Constitution Alignment)

- **No bypass in default behavior**: default runs must follow **probe → fixed‑size classification → mode selection → refinement**.
- **Determinism given seed**: same objective + bounds + seed + budget ⇒ identical decisions and best value (up to numeric tolerance) in the same environment.
- **Bounded overhead**: per‑eval policy work is O(1) and does not add extra objective calls.
- **Artifact auditability**: outputs are schema‑versioned and include per‑eval decisions and environment fingerprint.
- **Canonical validation environment**: local validation runs use `PYTHONNOUSERSITE=1` and the canonical env (internally `helios-gpu-118`), without making that env name a customer‑facing requirement.

### 4.3 Functional Requirements

**FR1 – Probe phase**
- Evaluate a fixed number of probe samples derived deterministically from `seed`, `bounds`, and `probe_ratio`.
- Record probe samples and objective values in artifacts (or a replayable digest).

**FR2 – Classification phase**
- Run a fixed‑size classification test (default: 50 samples) and produce:
  - `variance_score` (or equivalent scalar),
  - `variance_label ∈ {structured, chaotic}`,
  - the chosen `mode` for refinement.
- Classification must be **authoritative** for mode selection in default behavior.

**FR3 – Refinement modes**
- Support these refinement strategies as first‑class modes:
  - **Structured mode**: guided strategy that exploits the probe signal.
  - **Chaotic mode**: general‑purpose strategy (e.g., TPE‑like) suitable for rugged objectives.

**FR4 – Artifacts**
- Emit a schema‑versioned run artifact containing at minimum:
  - Inputs: bounds digest, budget, seed(s), probe_ratio.
  - Phase timings (probe, classify, refine, total).
  - Classification outputs (score, label, mode).
  - Per‑eval trace: phase, candidate, value, best‑so‑far, and any strategy decisions.
- Capture an environment fingerprint sufficient to reproduce results.

**FR5 – Determinism**
- All randomness must come from an explicit seeded RNG.
- Tie‑breaking must be stable.

---

## 5. Metrics & Evaluation Plan

### 5.1 Primary Metric

- **Median time‑to‑target** across a fixed seed suite, per objective and per use case.

### 5.2 Secondary Metrics

- Hit‑rate: fraction of seeds reaching the target within budget.
- Best value at fixed budget.
- Overhead: optimizer CPU time per eval (and total runtime) compared to baselines.

### 5.3 Baselines

- Random search.
- Pure Optuna TPE (or equivalent general‑purpose optimizer).

### 5.4 Minimum Benchmark Suite (MVP)

- **Fast sim tuning**:
  - Synthetic expensive smooth objectives (sleep‑injected) and at least one "sim‑like" structured function.

- **Sklearn tuning**:
  - At least one linear model objective (e.g., `SGDClassifier` / `SGDRegressor`).
  - At least one nonlinear model objective.

---

## 6. Risks & Open Questions

**R1 – Target thresholds**  
- [NEEDS CLARIFICATION] Exact target thresholds per benchmark objective for time‑to‑target comparisons.

**R2 – Structured vs chaotic thresholds**  
- [NEEDS CLARIFICATION] Numerical thresholds and rules for "structured" vs "chaotic" labels.

**R3 – Mode strategy variants**  
- [NEEDS CLARIFICATION] Which refinement strategies are in‑scope for MVP (one per mode vs more options behind config/flags).
- MVP defaults:
  - Structured mode: Nelder-Mead (robust, gradient-free, suited to smooth sims).
  - Chaotic mode: TPE (Tree-structured Parzen Estimator, standard for ML tuning).

---

## 7. Implementation & Packaging Constraints (Non‑functional)

- Core implementation MUST be a **Rust library crate** exposing the probe‑gated solver API.
- CLI MUST be a thin **Rust binary crate** that delegates to the core.
- SDKs (for example, Python) MUST be **thin bindings** over the same core, not reimplementing solver logic.
- Artifacts MUST be language‑agnostic (JSON) and used as the compatibility contract between surfaces.

### 7.1 Python Binding Strategy (Non‑functional)

- The Python SDK MUST use **PyO3 directly** over the Rust core crate (no ctypes or cffi wrapper over the C ABI for primary usage).
- A minimal C‑ABI crate MAY exist for non‑Python SDKs; it MUST remain small and stable and MUST NOT contain business logic.
- Objective functions implemented in Python MUST be invoked via a dedicated Python‑objective bridge that minimizes cross‑boundary overhead (for example by targeting expensive objectives or supporting batching where appropriate).

---

## 8. Deliverables (This Spec Only)

This spec defines **what** ArqonHPO v1 must do and how success is measured. Implementation details and task breakdown are deferred to Plan/Tasks documents.

- Finalized `spec.md` (this file), aligned with the ArqonHPO Constitution.
- A follow‑on `plan.md` describing:
  - Rust workspace layout (`core`, `cli`, `ffi`, `bindings/python`).
  - Config and artifact type definitions.
  - Benchmark harness and evidence pack locations.
- A follow‑on `tasks.md` breaking implementation into test‑first tasks mapped to FRs and user stories.
