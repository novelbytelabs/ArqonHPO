# POC-A: Machine-speed knob adaptation under drift (Tier-2) with deterministic Tier-1 safety

This POC demonstrates the **online** portion of the "Discovery offline, Adaptation online" architecture:

- **Data plane** (simulated): produces request-level telemetry under drifting conditions (load/data drift/hardware).
- **Tier 2 adaptive engine**: uses **SPSA** (2 evaluations/update, independent of knob count) to continuously tune *continuous knobs*.
- **Tier 1 executor**: deterministic safety gate that enforces:
  - allowlist of tunable knobs
  - hard bounds
  - max delta per apply
  - rate limits
  - rollback + snapback baseline on sustained constraint violations
- **No Omega**: no LLM observer; no online invention—just bounded adaptation.

## Quick start

```bash
python poc_a_knob_adaptation.py
```

Outputs:
- `out/poc_a_run.png` plot (loss/latency/error/knobs over time)
- `out/audit_log.jsonl` event-sourced audit stream
- `out/history.csv` digest history table

## Notes

This is a *systems POC* focused on architecture correctness and safety behavior.
It is intentionally lightweight and deterministic (seed-controlled).
