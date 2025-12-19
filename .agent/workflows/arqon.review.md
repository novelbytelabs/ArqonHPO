---
description: Fast, local review of staged/unstaged changes using Spec Kit artifacts when available (low-noise, evidence-based).
---

---
description: Fast, local review of staged/unstaged changes using Spec Kit artifacts when available (low-noise, evidence-based).
---

## User Input

```text
$ARGUMENTS
```

You MUST consider the user input before proceeding (if not empty).

## Goals

- Provide a high-signal local review while coding.
- Prefer correctness, determinism/perf, safety, and spec-drift over style nits.
- If you cannot cite evidence from the diff/context, phrase it as a QUESTION (not a claim).

## Steps

1. From repo root, run:
   - `.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`
   Parse FEATURE_DIR and AVAILABLE_DOCS. All paths must be absolute.

2. Collect diffs:
   - Unstaged: `git diff`
   - Staged: `git diff --staged`
   If both are empty, STOP and say: "No local changes to review."

3. Collect minimal context (only what’s needed):
   - `git status --porcelain`
   - `git diff --name-only`
   - If FEATURE_DIR exists and docs are available, read (if present):
     - constitution (or .specify/memory/constitution.md if that’s what your repo uses)
     - spec
     - plan
     - tasks

4. Produce a single markdown report with this exact structure:

### Summary (3 bullets max)
- ...

### Blockers (0–3)
- **[BLOCKER]** ... (include file(s) and the specific diff hunk or line reference if possible)

### High-risk concerns (0–5)
- **[RISK]** ... (cite evidence)

### Test gaps
- ...

### Spec/Plan drift
- What the artifacts require vs what the diff does.

### Questions (only if needed)
- ...

### Next actions (ordered)
1. ...

## Hard constraints (keep it useful)

- NO style nitpicks unless they violate an explicit rule/artifact.
- Cap total bullets to ~15.
- Do not suggest large refactors unless clearly risk-reducing and supported by evidence.