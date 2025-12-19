---
description: "Performance regression scout: identify hot-path risk in current changes (local or PR), suggest what to measure, and produce a perf-focused checklist (no indexing required)."
---

# /arqon.perf_guard

This is a **performance-first review workflow**. It does not try to fully benchmark everything; it flags likely perf hazards and produces the smallest credible measurement plan.

## What you can type with the command (optional)
In the same message as the slash command, you may include:
- `pr <num>` to analyze a specific PR diff
- `local` to analyze local changes (default if no PR is detected)
- `bench` to also run lightweight benches (only if repo supports it and you explicitly asked)
- `full` to broaden the checklist and measurement plan

If nothing is provided:
- Prefer local diff if there are local changes
- Otherwise, try current branch PR via `gh pr view`

---

## Safety / Guardrails
- Never run destructive commands.
- Never “optimize” code directly in this workflow.
- Only run benches if the user explicitly included `bench`.

---

## Step 0 — Determine target (local vs PR)

1. Capture local change status:
   - `git status --porcelain`
   - `git diff --name-only`
   - `git diff --name-only --staged`

2. Parse user message:
   - If contains `pr <num>` (or clear PR number), target PR
   - Else if local changes exist, target local
   - Else attempt PR for current branch (best-effort):
     - `command -v gh`
     - `gh auth status`
     - `gh pr view --json number -q .number`

If no target can be determined, STOP and ask the user to specify `pr <num>` or make local changes.

---

## Step 1 — Collect the diff and changed file list

### If targeting local
- Diff:
  - `git diff`
  - `git diff --staged`
- Files:
  - `git diff --name-only`
  - `git diff --name-only --staged`

### If targeting PR
- Verify GitHub remote:
  - `git config --get remote.origin.url` must be GitHub
- Collect:
  - `gh pr diff <PR>`
  - `gh pr view <PR> --json files` (best-effort; if not available, infer from diff)

---

## Step 2 — Identify performance-sensitive areas (heuristic, no index)

Build a “Perf Risk Map” using these signals:

### A) File path heuristics
Flag higher risk if changes touch:
- core runtime loops, scheduling, routing, queues, event bus
- serializers/deserializers, codecs
- networking, I/O boundaries
- alloc-heavy modules (collections, buffers)
- tight loops / inner kernels
- anything labeled “hot”, “fast”, “core”, “runtime”, “engine”, “bus”, “loop”

### B) Diff pattern heuristics
Search changed lines for:
- new allocations: `Vec::new`, `String::new`, `clone`, `to_string`, `format!`, `collect::<Vec`, `Box::new`
- logging in hot code: `log::`, `tracing::`, `println!`
- blocking calls on critical paths: `std::thread::sleep`, blocking I/O
- locks / contention: `Mutex`, `RwLock`, `Arc<Mutex`, `parking_lot::Mutex`
- async pitfalls: `.await` inside loops, unbounded task spawning
- algorithmic blowups: nested loops, repeated scans, `O(n^2)` patterns
- conversions: repeated parsing/encoding per message
- extra copying: `memcpy`-like patterns, buffer re-allocs
- cache-unfriendly changes: switching from arrays/slices to hashmaps, etc.

Use `rg` on the diff text if useful.

### C) “Behavior change” signals
Flag as perf-risk if the change:
- adds new work per request/message
- changes defaults to “more thorough” behavior
- increases sampling/logging verbosity
- introduces new validation in tight paths

---

## Step 3 — Determine what to measure (minimal measurement plan)

Create a measurement plan with:
- **Metric(s)** (throughput, latency p50/p99, allocations, CPU time)
- **Scenario(s)** (what input/load makes sense)
- **Baseline** (previous commit/tag/branch)
- **Comparison method** (before/after runs; stable machine if possible)

### Suggested baselines
- Local: `HEAD~1` or the merge-base with main
- PR: merge-base between head and base branch

### Example measurement suggestions (Rust)
- Throughput: messages/sec, requests/sec
- Latency: microbenchmark or representative integration test timing
- Allocation tracking: if you have tooling, suggest enabling it; otherwise note as follow-up
- Bench harness: `cargo bench` (only if user asked)

---

## Step 4 — Optional: run lightweight benches (`bench` only)

Only if the user included `bench`:

### Rust
If `Cargo.toml` exists:
- Run:
  - `cargo bench`
If too heavy, attempt to narrow if your repo supports bench filters (only if known; do not guess flags).

If benches are not present or clearly too expensive, STOP running benches and instead output “Bench run skipped; here’s the minimal manual plan.”

---

## Step 5 — Produce a perf-focused report (markdown)

Output with this exact structure:

### Perf risk verdict
- ✅ Low / ⚠️ Medium / ❌ High
- One-sentence rationale tied to evidence

### Hot-path touchpoints (files / modules)
- List changed files that look perf sensitive and why

### Risky patterns detected (evidence-based)
- Bullet list with snippets/pattern descriptions (no speculation without marking it)

### Measurement plan (smallest credible)
1. Baseline: ...
2. Measure: ...
3. Compare: ...
4. Acceptance criteria: ...

### If you have benches
- What to run and how to interpret it

### Suggested mitigations (only if needed)
- Specific, minimal changes that reduce risk (e.g., “move logging out of loop”, “pre-allocate buffer”, “avoid clone in hot path”)

### Questions (only when required)
- Ask at most 3 targeted questions.

---

## Quality bar (non-negotiable)
- No generic “optimize” advice.
- Everything must tie to the actual diff or known repo conventions.
- Prefer “measure this to confirm” over “this is slower” unless proven.
