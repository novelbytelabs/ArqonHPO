---
trigger: always_on
---

# ULTIMATE INTEGRITY COVENANT

This constitution is a **hard constraint** on all engineering work. It exists to prevent failure modes that destroy real systems:

- “Happy path” engineering that collapses under real inputs
- Pseudocode/placeholders presented as completion
- Checkbox tests that don’t model production reality
- Fake evidence (invented logs/benchmarks/screenshots/coverage/results)
- Unnamed “temporary” debt that becomes permanent
- Silent failures, silent security degradation
- Undocumented complexity and unreadable “clever” code
- Non-reproducible builds, nondeterminism, flaky verification
- Work products that are narrative-only, vague, or unverifiable

If a decision conflicts with this covenant, **the decision is wrong**.

---

## A. The Only Acceptable Meaning of “DONE” (8-Pillar Standard)

A change is **NOT DONE** until **all eight pillars** are true:

1. Implementation  
   - Real code (not pseudocode). Builds, runs, handles edge cases.  
   - Invalid states rejected; invariants enforced.  
   - Failure behavior defined: timeouts, retries, backpressure, cancellation, partial failures.

2. Verification  
   - Automated tests cover normal + failure + adversarial + concurrency/ordering behavior.  
   - Tests model production complexity (not toy inputs).  
   - Where appropriate: property tests, fuzzing, chaos/fault injection, regressions.

3. Documentation  
   - In-repo docs cover: architecture, usage, invariants, data contracts, “what can go wrong.”  
   - Operational notes: limits, failure modes, rollback strategy, safety constraints.  
   - Public behavior changes update docs and/or changelog.

4. Evidence  
   - Reproducible proof exists (CI artifacts, logs/traces, coverage, benchmarks).  
   - Evidence is attached or linkable to a specific commit/run.  
   - No evidence → the claim is false.

5. Traceability  
   - Each requirement/acceptance criterion maps to code + tests + docs + evidence.  
   - No orphan requirements. No orphan code. No untested requirements.

6. Operational Readiness  
   - Safe defaults, config validation, clear failure signals.  
   - Observability: structured logs + metrics + tracing (or equivalent breadcrumbs).  
   - Rollout/rollback where relevant; migrations reversible or explicitly irreversible.

7. Security & Safety Readiness  
   - Threat assumptions stated; privilege boundaries validated.  
   - Secrets not logged; sensitive data redacted.  
   - Fail-closed for safety/security modules (no silent “allow”).

8. Task Completeness  
   - Concrete task list (not vague bullets).  
   - Each task has acceptance criteria + a test hook + an evidence hook.

Rule: Declaring “done” without satisfying every pillar is deception.

---

## B. Anti-Half-Ass Rules (Merge-Blocking)

### B1. No Pseudocode-as-Deliverable
- Pseudocode may exist only as clearly labeled design notes.  
- Pseudocode cannot be the solution, cannot substitute for tests, and cannot justify completion.

### B2. No Placeholders / No Stubs / No “Later”
Forbidden in production paths: `TODO`, `FIXME`, `pass`, `todo!()`, empty handlers, commented-out behavior, “mock later,” “harden later,” “edge cases later.”

If incomplete behavior must exist temporarily, it must:
- be feature-flagged **OFF** by default,
- be isolated so it cannot affect production behavior,
- have a `TD-###` record with a hard TTL (see Debt Policy).

### B3. No Fake Data / Toy Inputs / Lazy Synthetics
- Tests/examples must look like production: realistic IDs, nested payloads, boundary sizes, malformed variants, weird unicode/whitespace, adversarial inputs.
- Ban list (unless explicitly testing these literals): `foo`, `bar`, `user_1`, `test123`, “hello world.”

### B4. No Happy Path Testing
For externally coupled features (network/storage/auth/scheduler/filesystem/deps), tests must cover:
- timeouts, retries, partial failures, malformed responses, permission failures,
- cancellation, overload/backpressure, out-of-order/duplicate events (when relevant).

### B5. No Silent Failures
- Swallowing errors is forbidden. Errors must be handled, logged with context, or propagated.  
- “Fallback to success” without explicit docs + tests is forbidden.

### B6. Warnings Are Errors
- Compiler/linter/formatter/typechecker warnings block merge. “Works on my machine” is irrelevant.

### B7. No Unbounded Risk
- Unbounded queues/memory/metric cardinality/retries/timeouts are forbidden.  
- If it can grow, cap it. If it can retry, budget it. If it can wait, time it out.

---

## C. Technical Debt Policy (Zero Debt Unless Named + Owned + Expiring)

Technical debt is **forbidden by default**. If debt must exist, it must be explicit, bounded, and temporary.

Debt is valid only if recorded as `TD-###` with:
- owner,
- scope + blast radius,
- why it exists,
- exit criteria (“debt removed when…”),
- remediation plan,
- hard TTL date,
- tests guarding the boundary so the debt cannot silently expand.

Rules:
- Debt without TTL is invalid.
- Debt past TTL blocks merge/release.
- “We’ll fix later” is not a plan.
- “Temporary” paths must have an explicit sunset mechanism.

---

## D. SDD + TDD Contract (Professional Standard)

### D1. Specification-Driven Design (for non-trivial changes)
A valid spec includes:
- intent + non-goals,
- falsifiable acceptance criteria,
- invariants (must-always-be-true),
- failure modes + expected behavior under each,
- compatibility rules (protocol/API/schema expectations),
- performance envelope + resource bounds (when relevant),
- security/privacy assumptions + constraints,
- operational concerns (observability, rollout/rollback, migrations).

If the spec is ambiguous, the first task is to produce falsifiable criteria.

### D2. Test-Driven Development Requirements
- Tests define behavior before/with implementation (TDD by default).  
- Refactors keep tests protecting behavior.  
- Every bug fix includes a regression test (fails pre-fix, passes post-fix).  
- Flaky tests are critical bugs: fix them, don’t ignore them.

---

## E. Verification Constitution (Realism + Adversarial + Failure-First)

Required test categories (as applicable):
- unit (pure logic, fast, no external deps),
- integration (real boundaries + dependency interactions),
- property-based (parsers/validators/protocol/config boundaries),
- fuzz (user-controlled input surfaces),
- concurrency/ordering (races, duplicates, replays, idempotency),
- chaos/fault injection (externally coupled behaviors),
- performance regression (hot paths or stated latency budgets).

Verification must explicitly test (as applicable):
- malformed inputs, partial reads/writes,
- timeout handling, retry policy, idempotency guarantees,
- auth failures + permission boundaries,
- overload/backpressure behavior,
- deterministic ordering assumptions (or explicit non-guarantees).

---

## F. The Claim Ledger (Mandatory Honesty)

Any claim like “works,” “done,” “fixed,” “secure,” “fast,” “compatible,” “production-ready” must be labeled:

- **Observed**: executed + evidence attached
- **Derived**: reasoned + assumptions listed + risks stated
- **Unverified**: not tested + minimal experiment provided

Presenting Derived/Unverified claims as Observed is lying.

---

## G. Minimum Acceptable Deliverable (Non-Negotiable Output Shape)

Any non-trivial work product must include:
- a concrete task list with acceptance criteria per task,
- a file-level plan (what files change/add/remove),
- implementation code,
- tests (including failure/adversarial coverage where relevant),
- documentation updates,
- an Evidence Pack (defined below).

If anything is incomplete, label it **Unverified** and provide the shortest experiment to verify it.

---

## H. Default Principles

If not explicitly covered here, default to the most stringent, resilient, and transparent posture; treat ambiguity as a Constitutional void requiring immediate formal amendment.

---

# XIV. ULTIMATE INTEGRITY ATTESTATION & EVIDENCE PACK

This section is the **merge/ship gate**. “Done” is a reproducible fact, not a feeling.

---

## 1) Merge/Ship Attestation (Required)

By merging or shipping, the author(s) and reviewer(s) attest:

- No placeholders exist in production paths (no TODOs/stubs/pseudocode-as-work/“later hardening”).
- No fake evidence is presented (no invented logs/benchmarks/screenshots/coverage/results).
- Verification is not happy-path-only for critical behaviors.
- No silent failure handling exists; errors are handled/logged/propagated with context.
- Warnings were treated as errors (clean lint/typecheck/compile).
- Any technical debt is recorded as `TD-###` with owner + TTL + exit criteria and is bounded by tests.
- All claims are labeled Observed/Derived/Unverified, and Observed claims have attached evidence.

If you cannot honestly attest to every item above, you must not merge/ship.

---

## 2) Evidence Pack (Attach or Link; Required)

A change is invalid without a reproducible Evidence Pack tied to a specific commit and reproducible by another engineer.

### 2.1 Build Proof
- CI run or local output showing: clean build, clean lint/typecheck/format, warnings treated as errors.

### 2.2 Test Proof
- Results for: unit, integration (if applicable), property/fuzz (if required), concurrency/ordering (if applicable).
- Short note listing what is not covered and why (explicitly).

### 2.3 Failure Matrix Proof
For each externally coupled feature, list:
- failure scenarios tested (timeouts, retries, malformed responses, auth failures, overload/backpressure, partial failures),
- test file(s) + test names (or equivalent pointers).

### 2.4 Traceability Proof (Truth Table)
Provide a mapping:
- requirement/acceptance criteria → implementation location(s) → test location(s) → documentation location(s) → evidence artifact(s).

Rule: No test → untested requirement. No requirement → suspicious test/code.

### 2.5 Runtime Proof (When Applicable)
- Example run logs showing normal behavior + at least one failure mode behaving correctly.
- Proof observability works: correlation IDs + metrics/traces (or equivalent breadcrumbs).

### 2.6 Performance / Resource Proof (When Relevant)
- Baseline numbers + method + environment.
- Regression guard (benchmark, threshold check, or documented budget).
- Proof of bounded behavior (caps, backpressure, shedding policy).

### 2.7 Reproduction Commands
- One-command verification (e.g., `make test`, `just test`, `npm test`, `cargo test`).
- Environment notes: pinned toolchains/deps, required services (compose/containers), seed control for deterministic tests.

---

## 3) Debt Register Enforcement (TD-###)

If any `TD-###` exists in the change:
- TTL date + owner are mandatory.
- The boundary is protected by tests (cannot silently expand).
- Exit criteria is concrete (“remove X path,” “delete Y flag,” “replace Z dependency,” etc.).
- Debt past TTL is a release/merge blocker.

---

## 4) Professional Review Checklist (Hard Questions Only)

Review must answer “yes” with evidence:

- Failure modes explicit (not assumed)?
- Tests realistic, adversarial, non-trivial (no lazy synthetics)?
- Concurrency/ordering hazards tested or explicitly ruled out?
- Production observability present (logs/metrics/traces/breadcrumbs)?
- Resource bounds explicit (timeouts, caps, retry budgets, queue bounds)?
- Code readable under pressure (3 AM outage standard)?
- Docs match behavior and constraints?
- Another engineer can reproduce the Evidence Pack from scratch?

If any answer is “no,” the change is not complete.

---

## 5) Claim Ledger Summary (Required When Stating Status)

If a deliverable claims completion/correctness, include:
- Observed claims: link evidence
- Derived claims: list assumptions + risks + how to verify
- Unverified claims: list the minimal experiment to verify

Rule: If it cannot be reproduced from the Evidence Pack, it is not true.  
Rule: If it is not true, it is not done.

---
