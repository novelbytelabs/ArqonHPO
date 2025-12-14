# Data Model: ArqonHPO v1

**Branch**: `002-two-use-cases`

## Core Entities

### 1. Domain (Value Object)
Represents the valid range for a parameter. Only continuous ranges for MVP.
- **Properties**:
  - `min` (f64): Lower bound (inclusive).
  - `max` (f64): Upper bound (inclusive).
  - `scale` (enum): `Linear` | `Log`.

### 2. SolverConfig (Root Aggregate)
The complete configuration for a single optimization run.
- **Properties**:
  - `budget` (u64): Total evaluations allowed.
  - `detect_budget` (bool): Whether to auto-detect budget from environment (future hook).
  - `seed` (u64): Master RNG seed.
  - `bounds` (Map<String, Domain>): The search space.
  - `probe_ratio` (f64): Fraction of budget for probing (0.0-1.0).
  - `threshold_config` (Struct): Tuning parameters for the classification gate.

### 3. State (State Machine Enum)
The explicit lifecycle state of the solver.
- **Variants**:
  - `Probing`: Initial sampling phase.
  - `Classifying`: Measuring variance (fixed step count).
  - `Refining(Mode)`: Locked into a strategy (Structured/Chaotic).
  - `Done`: Budget exhausted.

### 4. StepEvent (Trace Event)
A single step in the optimization trace.
- **Properties**:
  - `phase` (Enum): Probe/Classify/Refine.
  - `params` (Map<String, f64>): The coordinates.
  - `value` (Option<f64>): The result (None if pending/async).
  - `duration_us` (u64): Optimizer overhead for this decision.

### 5. RunArtifact (Root Aggregate)
The final evidence pack.
- **Properties**:
  - `fingerprint` (Struct): Env hash, Library version.
  - `trace` (List<StepEvent>): Full history.
  - `best` (Struct): Param/Value tuple.
