# Product Overview

ArqonHPO is built on a radical premise: **optimization is a control problem, not a search problem.**

## The Architecture of Speed

Standard HPO libraries treat your system as a black box function $f(x) \rightarrow y$. They pause the world, run the function, and wait.

ArqonHPO treats your system as a **flow**:

1.  **Probe (Tier 0)**: Low-discrepancy sampling scans the landscape.
2.  **Classify (Tier $\Omega$)**: Is the signal structured (physics) or chaotic (noise)?
3.  **Refine (Tier 2)**: The Adaptive Engine proposes safe deltas.
4.  **Enforce (Tier 1)**: The Safety Executor applies changes atomically.

[Read about the Architecture](architecture.md){ .md-button }

## Core Components

- **Safety Executor**: The gatekeeper. Enforces rate limits, rollback contracts, and value bounds.
- **Adaptive Engine**: The brain. Uses SPSA or Nelder-Mead to steer parameters.
- **Telemetry Ring**: Lock-free, allocation-free circular buffer for observation.
