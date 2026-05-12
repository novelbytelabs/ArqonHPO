# Arqon Zero Sealed-Test Boundary Policy

Version: 0.1.0

## Rule

Raw sealed/holdout test contents must not be exposed to PM AI, Coder AI, Helper/Codex, or Auditor AI.

## Allowed

- Sealed-runner aggregate result packets
- Signed sealed-runner verdicts
- Hashes of sealed fixtures
- Non-content metadata required for provenance

## Forbidden

- Reading raw sealed fixtures
- Printing raw sealed examples
- Training or tuning against sealed fixtures
- Selecting or modifying gates after sealed results
- Using sealed outputs for repair loops
- Treating development diagnostics as sealed certification

## Hard Fail

Any suspected sealed-test contamination triggers:

- REQUIRES_HUMAN_REVIEW
- development diagnostic only
- NOT SEALED-TEST CERTIFIED
- not promotable
- quarantine review
