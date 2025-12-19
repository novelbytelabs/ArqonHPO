---
description: "Review a GitHub PR (Spec-Kit aware): diff + CI + spec/plan/tasks drift check"
---

## Usage
- `/arqon.review_pr` (current branch PR)
- `/arqon.review_pr 123` (explicit PR number)
- `/arqon.review_pr 123 post` (also post a comment)

## Steps

1. **Verify tooling**
   - Run: `command -v gh`
   - Run: `gh auth status`
   - If either fails: STOP and explain what’s missing.

2. **Verify we’re in a GitHub repo**
   - Run: `git config --get remote.origin.url`
   - STOP unless remote is `https://github.com/...` or `git@github.com:...`.

3. **Locate Spec-Kit artifacts (if present)**
   - Run: `.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`
   - Parse:
     - FEATURE_DIR (absolute path)
     - AVAILABLE_DOCS
     - Path to tasks (absolute path)
   - If the script is missing, continue with PR review anyway, but add a “Spec-Kit artifacts not found” note.

4. **Determine PR number**
   - If the user supplied an integer, use it.
   - Else run: `gh pr view --json number -q .number`
   - If this fails, STOP and tell the user to pass a PR number.

5. **Collect PR data**
   - Run: `gh pr view <PR> --json number,title,author,baseRefName,headRefName,url,body,labels`
   - Run: `gh pr diff <PR>`
   - Run: `gh pr checks <PR>`

6. **Read artifacts (only if they exist)**
   - Constitution
   - spec
   - plan
   - tasks

7. **Produce a markdown review report**
   Use this exact structure:

### Verdict
- ✅ Ready / ⚠️ Needs changes / ❌ Blocked

### CI status
- Summarize `gh pr checks` in 3–6 bullets (what failed, what’s flaky, what’s missing)

### Summary of change (<= 6 bullets)
- ...

### Blockers (0–5)
- **[BLOCKER]** ... (must cite evidence from diff/artifacts/CI)

### High-risk concerns (0–8)
- **[RISK]** ...

### Test gaps / Verification notes
- ...

### Spec/Plan drift
- What the artifacts require vs what the PR does.

### Next actions (dependency ordered)
1. ...

8. **Optional: prepare a PR comment**
   Short version (verdict + top blockers + top next actions + CI failures).

9. **Posting behavior**
   - Only if the user included the word `post`:
     - Post comment via: `gh pr comment <PR> --body "<PASTE COMMENT BODY HERE>"`

## Review principles
- No style nits unless they violate an explicit repo rule/constitution/spec/plan.
- If uncertain, ask a question instead of asserting.
- Prefer correctness/determinism/safety over cosmetics.
