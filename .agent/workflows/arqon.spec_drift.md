---
description: "Spec drift checker: compare code changes (local or PR) against Spec-Kit artifacts (constitution/spec/plan/tasks) and report only mismatches, missing work, and scope creep."
---

# /arqon.spec_drift

This workflow is a **pure alignment gate**. It does **not** do general code review. It only answers:

- Did we build what the artifacts say?
- Did we accidentally build extra things?
- What evidence is missing to call this DONE?

## What you can type with the command (optional)

In the same message as the slash command, you may include:
- `pr 123` → compare PR 123 against artifacts
- `local` → compare local changes (default if local changes exist)
- `strict` → treat missing tasks/tests/docs as blockers (aligns with “DONE means evidence”)
- `post` → post a short drift summary comment to the PR (PR mode only)

If nothing is provided:
- Prefer local diff if there are local changes
- Otherwise try current-branch PR via `gh pr view`

---

## Safety / Guardrails
- Do not run destructive commands.
- Do not modify artifacts or code.
- Do not post to GitHub unless the user explicitly included `post`.

---

## Step 0 — Determine target (local vs PR)

1. Capture local state:
   - `git status --porcelain`
   - `git diff --name-only`
   - `git diff --name-only --staged`

2. Parse the user message:
   - If includes `pr <num>`, target that PR
   - Else if there are local changes, target local
   - Else try:
     - `command -v gh`
     - `gh auth status`
     - `gh pr view --json number -q .number`

If no target can be determined, STOP and ask the user to specify `pr <num>` or create local changes.

---

## Step 1 — Locate Spec-Kit artifacts (required for this workflow)

From repo root, run:
- `.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`

If this fails or the artifacts are missing:
- STOP and report: “Spec-Kit artifacts not available; can’t compute drift.”
- Include how to fix: run Spec Kit steps to generate spec/plan/tasks.

Parse:
- FEATURE_DIR (absolute path)
- AVAILABLE_DOCS list
- Tasks path (absolute path)

---

## Step 2 — Collect the change set

### Local mode
- `git diff`
- `git diff --staged`
- `git diff --name-only`
- `git diff --name-only --staged`

### PR mode
- Verify remote is GitHub:
  - `git config --get remote.origin.url`
- Collect:
  - `gh pr diff <PR>`
  - `gh pr view <PR> --json number,title,url,body,labels`

---

## Step 3 — Read artifacts (only those present)

Read, in priority order:

1. **Constitution**
2. **Spec** (what/why, acceptance criteria)
3. **Plan** (architecture/approach, constraints)
4. **Tasks** (dependency ordered checklist)

Extract:
- Acceptance criteria
- Non-negotiables (constraints, safety, determinism/perf rules)
- Explicit deliverables (tests, docs, benchmarks, migration notes)
- Task list items (each task becomes an “expected work unit”)

---

## Step 4 — Map “expected work” to “observed work” (best-effort)

Create two sets:

### Expected set (from artifacts)
- Features/behaviors promised (from spec acceptance criteria)
- Constraints and invariants (constitution + plan)
- Required evidence (tests/docs/benchmarks, “DONE” rules)
- Task checklist items (titles)

### Observed set (from changes)
- Changed modules/files (diff file list)
- New public API surfaces (exports, endpoints, CLI flags, config keys) if detectable
- New dependencies or configuration changes (lockfiles, manifests)
- Added/updated tests and docs
- Added telemetry/logging/flags/migrations if present

---

## Step 5 — Drift detection rules (the heart of this workflow)

Report ONLY these categories:

1. **Missing deliverables**
   - Acceptance criteria not obviously addressed
   - Tasks not reflected in changes
   - Required tests/docs/bench evidence missing

2. **Scope creep**
   - Behavior introduced that is not in spec/plan/tasks
   - New deps/config changes not called for

3. **Constraint violations (artifact-defined)**
   - Anything that conflicts with constitution or plan constraints
   - If uncertain, phrase as “needs confirmation” with a question

4. **Traceability gaps**
   - Example: spec says “add benchmarks” but no bench changes
   - Plan says “feature flag” but no flag present

### Strict mode
If user included `strict`:
- Treat missing required evidence as **BLOCKER drift**.

Otherwise:
- List as “needs follow-up”.

---

## Step 6 — Produce the drift report (markdown)

Output with this exact structure:

### Drift verdict
- ✅ Aligned / ⚠️ Mostly aligned (follow-ups) / ❌ Drift detected (blocked)

### Missing items (from spec/plan/tasks)
- **[BLOCKER]** ... (strict mode) OR **[FOLLOW-UP]** ...

### Scope creep (unexpected additions)
- **[CREEP]** ...

### Constraint risks
- **[RISK]** ... (must cite artifact clause or plan detail)

### Evidence checklist
- Tests: present/missing + what’s missing
- Docs: present/missing + what’s missing
- Bench/perf proof: present/missing + what’s missing
- Migration/rollback: present/missing + what’s missing

### Next actions (dependency ordered)
1. ...
2. ...

### Questions (only if necessary)
- Max 3, targeted.

---

## Step 7 — Optional PR comment (PR mode + `post` only)

If in PR mode and the user included `post`:

- Post a short comment containing:
  - Drift verdict
  - Top 3 missing items
  - Top 3 next actions

Command:
- `gh pr comment <PR> --body "<PASTE COMMENT BODY HERE>"`

If posting fails, show the error and stop.

---

## Quality bar (non-negotiable)

- Do not do generic code review.
- Everything must reference: spec/plan/tasks OR the observed diff.
- If artifacts are missing, stop rather than hallucinate requirements.
