# Safety Presets and Rollback

## Guardrails Presets

The hotpath safety executor exposes guardrails presets for tuning safety boundaries:

- `conservative`
- `balanced`
- `aggressive`

JSON examples live in `examples/safety_presets/`.

## Rollback Policy

Rollback policy defines thresholds for reverting configuration changes. A default template is in:

`examples/safety_presets/rollback_policy.json`

Key fields:

- `max_consecutive_regressions`
- `max_rollbacks_per_hour`
- `min_stable_time_us`
