# Arqon Zero Audit Replay Policy

Version: 0.1.0

## Auditor Requirements

Auditor AI must independently verify:

- PM spec completeness
- GatePlan freeze before execution
- Helper/Codex command evidence
- Patch boundary
- Hash chain
- Claim/evidence alignment
- Status language preservation
- Sealed/holdout boundary

## Replay Verdicts

- AUDIT_PASS_DIAGNOSTIC_ONLY
- AUDIT_PASS_REQUIRES_HUMAN_REVIEW
- AUDIT_FAIL_HARD
- AUDIT_FAIL_REPLAY_MISMATCH
- AUDIT_FAIL_CONTAMINATION_RISK
- AUDIT_INCOMPLETE

## Rule

Audit pass does not equal promotion.

Only a human-signed promotion manifest can promote.
