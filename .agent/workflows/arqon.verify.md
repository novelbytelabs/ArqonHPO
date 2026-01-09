---
description: "Run a high-signal local verification suite (fast by default, comprehensive on demand) before pushing or opening a PR."
---

# /arqon.verify

A “developer-grade preflight” that catches 80–95% of issues *before* CI. It’s intentionally **low-noise**, **evidence-based**, and **repeatable**.

## Usage

- `/arqon.verify`  
  Fast preflight (recommended default).

- `/arqon.verify full`  
  More thorough: broader tests + (optional) benches + deeper checks.

- `/arqon.verify changed`  
  Try to target only changed packages/modules (best-effort).

- `/arqon.verify fix`  
  Apply safe auto-fixes (formatters) **only**, then rerun fast preflight.

- `/arqon.verify bench`  
  Run lightweight performance smoke checks (not a full benchmark campaign).

You can combine: `fix full`, `changed bench`, etc.

---

## Safety / Guardrails

- Treat the working tree as the source of truth.
- **Never** run destructive commands (e.g., `git reset --hard`, deleting files).
- Auto-fix mode (`fix`) is limited to formatters (e.g., `cargo fmt`, `ruff format`) and must not change semantics.

---

## Step 0 — Quick repo context

1. Show current state:
   - `git status --porcelain`
   - `git rev-parse --abbrev-ref HEAD`

2. Record what you’re about to validate:
   - `git diff --stat`
   - `git diff --stat --staged`

If there are **no changes** (staged or unstaged), STOP and say: “No local changes to verify.”

---

## Step 1 — Decide the verification scope

1. Parse `$ARGUMENTS`:
   - If contains `full`: run extended suite
   - If contains `changed`: attempt targeted suite
   - If contains `bench`: include perf smoke checks
   - If contains `fix`: run formatters first

2. Identify changed files (used for “changed” scope):
   - `git diff --name-only`
   - `git diff --name-only --staged`

If `changed` is requested but no changed files are detected, fall back to the default fast suite.

---

## Step 2 — Spec-Kit awareness (optional, but awesome)

If the repo contains Spec Kit scripts, run:

- `.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`

If it succeeds:
- Note FEATURE_DIR and AVAILABLE_DOCS.
- If tasks/spec/plan exist, skim for any explicit “verification requirements” (e.g., “run benches”, “update docs”, “add tests”) and incorporate them into the checklist below.

If the script does not exist or fails:
- Continue normally; do **not** treat it as an error.

---

## Step 3 — Apply safe auto-fixes (only if `fix` requested)

Run these only if the corresponding files/tools exist:

### Rust (Cargo)
- If `Cargo.toml` exists:
  - `cargo fmt`

### Python
- If `pyproject.toml` exists:
  - If `ruff` is available: `ruff check --fix .`
  - If `ruff` supports formatting: `ruff format .`

### JS/TS
- If `package.json` exists:
  - Prefer: `npm run format` (or `pnpm run format` / `yarn format` depending on your repo)
  - If unknown, do not guess; just note “format script not found”.

After auto-fix, re-run:
- `git diff --stat`
- `git diff --stat --staged`

---

## Step 4 — Fast preflight suite (default)

Run what applies:

### A) Rust (Cargo)
If `Cargo.toml` exists:

1. Toolchain sanity:
   - `rustc --version`
   - `cargo --version`

2. Format check:
   - `cargo fmt --check`  
   If this fails, it’s a BLOCKER.

3. Lints:
   - `cargo clippy --all-targets --all-features -- -D warnings`  
   If this fails, it’s a BLOCKER.

4. Tests (fast):
   - `cargo test --all-features`  
   If this fails, it’s a BLOCKER.

### B) Python
If `pyproject.toml` exists (or `requirements*.txt` exists):

1. Lint/format checks (best-effort):
   - If `ruff` exists: `ruff check .`
   - If `python -m compileall` is relevant: `python -m compileall .`

2. Tests:
   - If `pytest` exists: `pytest -q` (or your repo’s standard)  
   If this fails, it’s a BLOCKER.

### C) JS/TS
If `package.json` exists:

1. Install integrity (best-effort):
   - Prefer lockfile-aware commands your repo uses (`npm ci`, `pnpm i --frozen-lockfile`, etc.)
   - If unknown, do not guess; rely on `npm test`/repo scripts.

2. Lint/test:
   - `npm test` (or repo standard)
   - `npm run lint` (if present)
   Failures are BLOCKERS.

### D) Repo hygiene checks (always)
1. Search for obvious foot-guns in changed lines (best-effort):
   - `git diff | rg -n "TODO\\(|FIXME|HACK|panic!\\(|unwrap\\(|console\\.log\\(|print\\("`  
   This is not automatically a blocker, but call out anything suspicious.

2. Secrets smell-check (best-effort):
   - `git diff | rg -n "(API_KEY|SECRET|TOKEN|PRIVATE_KEY|BEGIN (RSA|OPENSSH) PRIVATE KEY)"`  
   If hits are found, treat as a BLOCKER until proven safe.

---

## Step 5 — “Full” suite (only if `full` requested)

### Rust extras
If `Cargo.toml` exists:

- Wider tests:
  - `cargo test --all-targets --all-features`

- Documentation build (fast):
  - `cargo doc --no-deps`

- MSRV or toolchain constraints (if your repo enforces them):
  - Run your repo’s standard MSRV check command if documented.

### Python extras
- Type checks if your repo uses them:
  - `mypy .` (or your configuration)
- More verbose tests:
  - `pytest -q` + any integration markers your repo expects

### JS/TS extras
- Typecheck:
  - `npm run typecheck` (if present)
- Build:
  - `npm run build` (if present)

---

## Step 6 — Performance smoke checks (only if `bench` requested)

This is intentionally a “smoke test”, not a full benchmark pipeline.

### Rust
If benches exist (best-effort detection):
- `cargo bench`  
If this is too heavy, run a subset if your repo supports it (documented bench filters).

If you have known hot-path invariants (allocations, determinism):
- Call out anything in the diff that would likely change perf characteristics (allocs in tight loops, logging in hot code, unnecessary clones).

---

## Step 7 — Produce a single, high-signal report

Output a markdown report with this structure:

### Overall status
- ✅ Pass / ❌ Fail (blocked)

### What ran
- List commands executed (grouped by language/tooling)

### Blockers
- Bullet list of failures with the exact failing command + key error line(s)

### Warnings / follow-ups
- Items that aren’t failing but should be reviewed (TODOs, suspicious patterns, missing tests)

### Suggested next action
- 1–3 steps max (e.g., “Fix clippy warnings”, “Add test for edge case X”, “Rerun /arqon.verify full”)

If anything failed: explicitly say “Do not push/PR until blockers resolved.”

---

## Notes for customizing this workflow (recommended)

If your repo already has a “one true command” (e.g., `./scripts/verify.sh` or `just verify` or `make check`):
- Prefer calling that, and keep the above as a fallback.
- The goal is consistency: one command you trust, everywhere.
