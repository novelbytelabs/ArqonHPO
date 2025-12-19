---
description: "Generate crisp, customer-ready release notes from git history + (optional) PR context + Spec-Kit artifacts. Includes breaking changes, upgrade notes, and verification checklist."
---

# /arqon.release_notes

Creates a high-quality release note draft that is **useful to customers** and **useful to you** (upgrade notes + verification).

## What you can type with the command (optional)
In the same message as the slash command, you *may* include:
- a version/tag (e.g. `v0.12.0`)
- a range (e.g. `v0.11.0..HEAD` or `v0.11.0..v0.12.0`)
- a PR number (e.g. `PR 123`)
- the word `write` to save the draft to a file (opt-in)

If nothing is provided, it will generate notes from **last git tag → HEAD**.

---

## Safety / Guardrails
- Do not push tags.
- Do not publish releases.
- Do not modify files unless the user explicitly included `write`.

---

## Step 0 — Establish context (repo + intent)

1. From repo root:
   - `git status --porcelain`
   - `git rev-parse --abbrev-ref HEAD`

2. Parse the user message for any of:
   - `PR <num>` (or a bare number that clearly looks like a PR id)
   - a tag like `vX.Y.Z`
   - a range like `<tag>..HEAD` or `<tag>..<tag>`
   - `write`

If ambiguous, proceed with the default (last tag → HEAD) and note assumptions.

---

## Step 1 — Determine the change range

### Default path (recommended)
1. Find the most recent tag:
   - `git describe --tags --abbrev=0`
   If this fails (no tags), fall back to:
   - range: last 50 commits on current branch

2. Set range:
   - `<LAST_TAG>..HEAD`

### If the user provided a range
Use it as-is after minimal validation (it should be a valid git revision range).

### If the user provided a target tag/version
- If the tag exists:
  - `<PREVIOUS_TAG>..<TARGET_TAG>` if both exist
- Else:
  - treat it as a “draft for upcoming release” and use `<LAST_TAG>..HEAD`

---

## Step 2 — Collect evidence (git, and optionally gh)

### Always collect (git)
1. Commit list (no merges by default):
   - `git log --no-merges --date=short --pretty=format:"%h %ad %s" <RANGE>`

2. Full messages for better parsing:
   - `git log --no-merges --pretty=format:"%h%n%s%n%b%n---" <RANGE>`

3. Diffstat (for scope awareness):
   - `git diff --stat <RANGE>`

4. Optional: include merges if your workflow uses “merge commits as story”:
   - `git log --merges --pretty=format:"%h %s" <RANGE>`

### If user referenced a PR (and gh is available)
1. Verify:
   - `command -v gh`
   - `gh auth status`

2. Fetch PR metadata:
   - `gh pr view <PR> --json number,title,url,author,body,labels,baseRefName,headRefName`

3. Fetch PR commits (best-effort; depends on gh version):
   - `gh pr view <PR> --json commits`
   If unavailable, skip without failing.

---

## Step 3 — Pull Spec-Kit artifacts (optional but preferred)

If present, run:
- `.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`

If it succeeds:
- Identify and read available:
  - constitution
  - spec
  - plan
  - tasks

Use these artifacts to:
- extract “what the feature is” in plain language
- identify explicitly required changes (docs, tests, migration notes)
- ensure the release notes match the intended scope

If missing, continue and add a note: “No Spec-Kit artifacts detected for this range.”

---

## Step 4 — Classify changes into customer-relevant buckets

Using the commit subjects/bodies (and PR body if available), assign entries to:

1. **New features**
2. **Fixes**
3. **Performance**
4. **Stability / reliability**
5. **Developer experience**
6. **Documentation**
7. **Security**
8. **Internal refactors** (only mention if it affects users)
9. **Breaking changes** (high priority)

### Breaking-change detection heuristics (best-effort)
Flag as potential breaking change if any commit message/body mentions:
- “breaking”, “deprecate”, “remove”, “rename”
- API signature change hints
- config/env var changes
- migration steps
- default behavior changes

If uncertain, list it under “Potential breaking changes (needs confirmation)”.

---

## Step 5 — Produce the draft release notes (markdown)

Output **two** versions:

### A) Customer-facing notes (tight)
- Clear, short, non-jargony
- Avoid internal implementation details unless they matter
- Grouped by category
- Include links only if you have them (e.g., PR URL); otherwise omit

### B) Maintainer notes (operational)
- Migration / upgrade notes
- Rollback notes (if applicable)
- Verification checklist (how to confirm the release is good)
- “Known issues / follow-ups”

Use this exact structure:

## Release <VERSION OR "Unreleased">
### Highlights
- (3–5 bullets max)

### Added
- ...

### Changed
- ...

### Fixed
- ...

### Performance
- ...

### Security
- ...

### Deprecations / Breaking changes
- ...

### Upgrade / Migration notes
- ...

### Verification checklist
- [ ] ...
- [ ] ...

### Notes for maintainers
- CI/CD considerations, bench comparisons, artifact checks, etc.

If you cannot determine a version, use:
- `Release Unreleased (<RANGE>)`

---

## Step 6 — Optional write-to-file behavior

Only if the user message included `write`:

1. Write a file such as:
   - `RELEASE_NOTES_DRAFT.md`
2. Ensure it includes the exact output from Step 5.
3. Report what file was written.

If `write` was not requested, do not create or modify any files.

---

## Quality bar (non-negotiable)
- Prefer **usefulness** over completeness.
- If you make a claim, it must be supported by commit/PR/spec evidence.
- If uncertain, mark it as “needs confirmation” and ask a single targeted question at the end.
