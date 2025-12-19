---
description: "Changelog + versioning guard: ensure the change includes the required release metadata (version bumps, changelog entries, migration notes, docs updates) based on repo conventions and Spec-Kit artifacts."
---

# /arqon.changelog_guard

This workflow is a “release hygiene gate” that answers:
- Did we remember the **boring but critical** release metadata?
- Are there hidden breaking changes without upgrade notes?
- Does the change meet your “DONE means evidence” standard?

It does **not** write files unless explicitly asked.

## What you can type with the command (optional)

In the same message as the slash command, you may include:
- `pr 123` → check PR 123
- `local` → check local changes (default if local changes exist)
- `strict` → treat missing required release metadata as blockers
- `write` → create a draft changelog entry file/snippet if missing (opt-in)
- `post` → post a short “hygiene checklist” comment to the PR (PR mode only)

If nothing is provided:
- Prefer local diff if there are local changes
- Otherwise try current-branch PR via `gh pr view`

---

## Safety / Guardrails
- Do not publish releases.
- Do not tag versions.
- Do not edit files unless the user explicitly included `write`.
- Do not post to GitHub unless the user explicitly included `post`.

---

## Step 0 — Determine target (local vs PR)

1. Capture local state:
   - `git status --porcelain`
   - `git diff --name-only`
   - `git diff --name-only --staged`

2. Parse user message:
   - If includes `pr <num>`, target PR
   - Else if local changes exist, target local
   - Else attempt PR for current branch:
     - `command -v gh`
     - `gh auth status`
     - `gh pr view --json number -q .number`

If no target can be determined, STOP and ask for `pr <num>` or local changes.

---

## Step 1 — Collect file lists + diffstat

### Local
- `git diff --name-only`
- `git diff --name-only --staged`
- `git diff --stat`
- `git diff --stat --staged`

### PR
- Confirm GitHub remote:
  - `git config --get remote.origin.url`
- Collect:
  - `gh pr diff <PR>`
  - `gh pr view <PR> --json number,title,url,body,labels`

Infer changed file list from the diff if `gh` doesn’t provide it directly.

---

## Step 2 — Detect repo conventions (best-effort)

Check for the presence of common release metadata files:

### Changelog files
Look for:
- `CHANGELOG.md`
- `CHANGES.md`
- `RELEASE_NOTES.md`
- `docs/` release notes conventions (repo-specific)

### Version sources
Look for one or more:
- Rust: `Cargo.toml` (package version), workspace conventions
- Python: `pyproject.toml`, `setup.cfg`, `__init__.py` version
- Node: `package.json`
- Custom: `VERSION`, `.version`, etc.

### Migration / upgrade docs
Look for:
- `MIGRATIONS.md`, `UPGRADING.md`, `docs/migrations/*`

If none of these exist, do not guess; instead, report “No obvious changelog/version convention detected” and proceed with minimal checks.

---

## Step 3 — Pull Spec-Kit artifacts (optional but recommended)

If present:
- `.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`

If it succeeds, read relevant artifacts and extract:
- Any explicit requirement for docs/notes/migration/bench evidence
- Any “breaking change” warnings
- Any acceptance criteria requiring documentation or user-facing changes

If missing, continue.

---

## Step 4 — Changelog guard checks

### A) Changelog entry present?
If a changelog file exists:
- Check whether it was modified in this change set.
- If not modified, mark as:
  - **BLOCKER** in `strict` mode
  - **FOLLOW-UP** otherwise

If no changelog file exists:
- Report that the repo may not maintain changelogs, and skip this check.

### B) Version bump present?
If a version file exists (e.g., Cargo.toml / package.json / pyproject):
- Check whether it was modified.
- If not modified, decide based on heuristics:
  - If change appears user-facing (new feature / behavior change / fix), suggest a bump.
  - If internal-only (docs/refactor/tests), bump may not be needed.

Do not insist unless `strict` and the repo convention clearly requires it.

### C) Breaking change detection
Scan diff text for signals:
- removed/renamed exported APIs
- config key changes
- default behavior change
- schema/migration changes
- CLI flag changes

If likely breaking:
- Require an “Upgrade / Migration notes” section somewhere (changelog or upgrading docs).
- If missing:
  - **BLOCKER** in strict mode, otherwise **HIGH PRIORITY FOLLOW-UP**

### D) Documentation hygiene
If change is user-facing:
- Check for updates in:
  - README
  - docs/
  - examples/
If none changed, suggest a doc follow-up.

### E) Evidence hygiene
If the spec/constitution says “DONE requires X evidence”:
- Confirm tests/docs/bench/migration notes appear in the change set.
- If not, flag as drift/hygiene gap.

---

## Step 5 — Produce the report (markdown)

Output with this exact structure:

### Hygiene verdict
- ✅ Good / ⚠️ Missing follow-ups / ❌ Blocked (strict)

### Detected conventions
- Changelog: <file or none>
- Version source: <file(s) or none>
- Upgrade/migration docs: <file(s) or none>

### Missing items
- **[BLOCKER]** ... (strict) OR **[FOLLOW-UP]** ...

### Potential breaking changes
- **[BREAKING?]** ... (with evidence)
- Required notes: present/missing

### Recommended changelog entry (draft)
- A short “Added/Changed/Fixed” bullet draft tailored to the diff/spec

### Next actions (dependency ordered)
1. ...
2. ...

### Questions (only if needed)
- Max 3 targeted questions

---

## Step 6 — Optional write behavior (`write` only)

If the user included `write`:
- If changelog exists and wasn’t updated:
  - Append (or propose) a well-formed entry in the correct section (do not guess format; match existing style).
- If no changelog exists:
  - Create `CHANGELOG_DRAFT.md` with the drafted entry and a note to integrate it into the project’s preferred format.

Never bump versions automatically in this workflow (too risky); only suggest the version bump.

---

## Step 7 — Optional PR comment (`post` only)

If in PR mode and the user included `post`:
- Post a short checklist comment:
  - Hygiene verdict
  - Top 3 missing items
  - If breaking: “Upgrade notes required”

Command:
- `gh pr comment <PR> --body "<PASTE COMMENT BODY HERE>"`

If posting fails, show the error and stop.

---

## Quality bar (non-negotiable)
- Prefer repo conventions over generic rules.
- If conventions are unclear, present options rather than asserting.
- Keep it actionable and short.
