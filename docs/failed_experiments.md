# Failed Experiment: Multi-Start Nelder-Mead (2025-12-14)

## Objective
Use K parallel Nelder-Mead instances from diverse seeds to better explore structured, multimodal landscapes (like Rosenbrock).

## Implementation
- `MultiStartNM` struct managing K instances.
- Dimension-aware adaptive K (`max(80, 25*(d+1))`).
- Triage-Commit schedule (20 evals triage -> pick best).

## Failure Mode
- **Regressed on Sphere/Rosenbrock**: Gap vs Optuna widened from ~2.5x to ~4.5x.
- **Starvation**: Even with adaptive K, the available budget (e.g. 200) was split too thin.
- **Triage Failure**: Short triage budgets (20 evals) were insufficient for NM to differentiate "good" runs from "bad" runs on shifted landscapes.

## Reversion Strategy
- Rolled back to **Single-Start Nelder-Mead** for Structured landscapes.
- Retained **TPE + 30% Spice** for Chaotic landscapes (Rastrigin win).
- Implemented **Class-Dependent Probe Spice** to prevent high-noise poisoning of structured runs.

## Future Recommendations
To retry Multi-Start NM safely:
1.  **Strict Minimum Budget**: Only enable K>1 if `budget > 500`.
2.  **Better Triage**: Use a cheaper proxy (Coordinate Descent?) for triage instead of full NM steps.
3.  **Simplex Regularization**: Ensure initial simplex is regular and scaled correctly to the domain, regardless of probe seeds.
