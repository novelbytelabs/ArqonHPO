# ArqonBus Constitution

This document defines the **non-negotiable principles** that govern how ArqonBus is designed, evolved, and maintained.

It exists to protect ArqonBus from accidental bloat, regression, silent breakage, and “clever” shortcuts that erode trust.

If a decision conflicts with this constitution, **the decision is wrong**.

> **Spec Kit Note:** This constitution is the hard sandbox for all `/speckit.*` commands.  
> Specs, plans, and tasks **must not** violate the constraints in Sections II–XI.

---

# I. Vision and Scope

## 1. The Vision

**ArqonBus is the Nervous System for the Intelligence Age.**

For the last two decades, real-time infrastructure was built for **Humans**—biological entities with 200ms reaction times, intermittent connectivity, and single-threaded focus.

We are entering the era of **Operators**—silicon or physical entities with microsecond reaction times, persistent connectivity, and swarm intelligence.

ArqonBus is the infrastructure shift required to support this transition. It is a **Coordination Fabric** designed to treat:

1. **Humans** (Chat, Collaboration),
2. **Devices** (IoT, Sensors), and
3. **Operators** (AI Agents, Physical Rigs, Computational Entities)

…as first-class citizens on a single, unified bus.

Our vision is to provide the **Substrate** where the Agent Economy lives—a place where safe, high-velocity, and stateful coordination happens faster than the speed of human thought. We are **Substrate-Agnostic**, treating diverse compute substrates as peers.

## 2. The Scope

To achieve this vision, we must be ruthless about what ArqonBus **is** and what it **is not**.

### 2.1 In Scope (The Core Product)

ArqonBus is **Transport and Coordination Infrastructure**. It is responsible for:

* **The Physical Layer:** Managing millions of concurrent TCP/QUIC/WebSocket connections with extreme efficiency (The Shield).
* **The Routing Layer:** Deliberating how messages move from point A to point B, whether via deterministic topics or semantic intent (The Spine).
* **The State Layer:** Maintaining the "Liveness" of the system—Presence, Room State, and Permissions (The Brain).
* **The Safety Layer:** Enforcing policy at the edge via Wasm (The Overseer). We do not decide what is safe; we provide the engine to enforce the user's definition of safety.
* **The Temporal Layer:** Providing access to the stream history for replay, debugging, and auditing (Time Travel).

### 2.2 Out of Scope (The Boundaries)

ArqonBus is **not an Application Runtime**.

* **We are not the AI:** ArqonBus routes prompts to LLMs; it does not host the LLM itself (except for lightweight embedding models for routing).
* **We are not the Database:** ArqonBus stores *Configuration* and *Message Logs*, but it is not a general-purpose replacement for your primary data warehouse.
* **We are not the Workflow Engine:** ArqonBus passes the signal that triggers a workflow; it does not execute the long-running business logic (e.g., generating a PDF or transcoding video). That belongs in external workers.

## 3. The Strategic Horizon

We define our evolution in three distinct epochs. Engineering decisions must align with the current epoch while reserving capacity for the next.

* **Epoch 1: The Foundation (Infrastructure).**
    * *Focus:* Unrivaled WebSocket performance, Multi-tenancy, and Developer Experience.
    * *Goal:* Be the best "Bus for Humans."
* **Epoch 2: The Platform (Programmability).**
      * *Focus:* Wasm Edge Hooks, Traffic Mirroring, and Schema Governance.
      * *Goal:* Be the most developer-friendly real-time platform.
* **Epoch 3: The Singularity (Intelligence).**
     * *Focus:* Semantic Routing, Agent Identity, and Swarm Consensus.
     * *Goal:* Be the Operating System for Multi-Agent Systems.

---

# II. Core Principles

This section defines the engineering laws that govern ArqonBus. These are not guidelines; they are constraints. Code that violates these principles will be rejected during Architectural Review.

### 1. Architectural Invariance (The Voltron Pattern)

The system is composed of four non-negotiable layers. **Strict adherence** to the **Layered Design** is required to prevent coupling and ensure scalability.

**The Layers:**

* **The Shield (Edge):** Responsible for connection termination, protocol normalization, and programmable safety. It holds **Zero Business State**.
* **The Spine (Bus):** The sole transport for all internal traffic and decoupling mechanism.
* **The Brain (Control):** Responsible for complex state, presence, authorization, and system self-healing. It optimizes system modes based on real-time **Churn Dynamics**.
* **The Storage (State):** The source of truth for configuration (Durable) and ephemeral counters (Hot).

**The Bypass Ban:**

No layer may reach around another because it is "convenient."

* **Forbidden:** The Shield directly accessing the Database.
* **Forbidden:** The Brain directly handling a TCP socket.
* **Forbidden:** Services communicating via direct HTTP/RPC calls. All internal traffic must traverse the Spine.
* **Service Bridging:** External services (e.g., REST/GRPC legacy apps) must not touch the Bus directly. They must be wrapped in a **Native Operator Shim** (Digital Proxy).

### 2. Statelessness & State Explicit-ness

To ensure the system can self-heal and scale horizontally, we adhere to a **Stateless Where Possible** philosophy.

* **Process Ephemerality:** Any individual process (Rust Gateway node, Elixir Control node) must be able to crash and restart without data loss.
* **State Locality:** Long-lived state belongs in **Postgres**. Hot, shared state belongs in **Valkey**.
* **Memory Hygiene:** In-memory state is permitted *only* in the Elixir Brain (for supervision trees/presence) and must be reconstructible from the persistent store or the message bus upon restart.

### 3. Protocol Sovereignty

We enforce a strict separation between machine-efficient communication and human-readable debugging.

**Protobuf on the Wire:**

All high-volume, internal, and agent-to-agent traffic must use **Protocol Buffers**. The `.proto` definitions are the single source of truth for the system.

**JSON Reservations:**

JSON is prohibited in the hot path. It is reserved exclusively for:

* Human-facing Administrative APIs (REST/GraphQL).
* Debug Consoles and Dashboards.
* Serialized views of internal state/telemetry for operators.

**Advisory Metadata:**

The protocol supports "Advisory" fields (e.g., `capabilities.*`, `fabric_family`) which are descriptive hints, not prescriptive contracts. These fields do not trigger strict schema validation failures if unknown/mismatched.

### 4. Future-Proofing Hooks (The Moonshot Mandate)

To enable the transition to Multi-Agent Systems (MAS) without requiring a v2.0 rewrite, v1.0 must reserve capacity for future intelligence features.

* **Identity Hook:** Authentication structures must support `Capabilities` lists (e.g., `["web_search", "code_gen"]`) alongside standard Roles.
* **Routing Hook:** Message Envelopes must reserve fields for `Semantic Vectors` (embeddings) and `Intents`, even if unused in v1.
* **Middleware Hook:** The Edge Gateway must implement a "Chain of Responsibility" pattern to allow the insertion of Wasm-based Safety Policies (The Overseer) later.
* **Physics Hook:** The architecture must reserve interfaces for **Phonic ROM** (BowlNet) integration, treating analogue transforms as native co-processing steps.
* **Pulse Hook:** The system must support **NVM Program Capsules** (Pulse Seeds) as a first-class message type, enabling the "Universal Decoder" pattern.
* **Pulse Certificates:** NVM Programs are passed by reference via **Pulse Certificates**, enabling the bus to orchestrate heavy analog compute without bandwidth saturation.
* **Superposition Hook:** Routing logic must allow for "late-binding" decisions (Schrödinger’s Packet), where message destination is determined by real-time load/affinity rather than static topics.
* **Teleportation Hook:** The bus must support atomic **State Transfer** jobs to move heavy execution contexts (Quantum Registers / NVM Snapshots) between nodes.
* **SAM Interfaces:** Agent Operators should expose standard capability descriptors (In/Out/Store/Create/Control) to facilitate automated composition.

### 5. Semantic Versioning & Compatibility

We adhere to strict Semantic Versioning regarding the **Protocol** and **Public API**.

**Versioning Rules:**

* **MAJOR:** Breaking changes to the Envelope, Command definitions, or Core Behavior.
* **MINOR:** New commands, additive fields (optional), new telemetry, or non-breaking logic enhancements.
* **PATCH:** Bug fixes, performance improvements, and clarifications.

**Stealth Ban:**

There shall be no "stealth" breaking changes in MINOR or PATCH versions. Ever.

**Backwards Compatibility:**

* **The Golden Rule:** A v1.1 Server must support a v1.0 Client.
* **Field Deprecation:** Fields must be marked `deprecated` in the Protobuf definition for at least one Minor cycle before removal in a Major release.
* **Graceful Degradation:** If a Client sends a valid message with unknown (new) fields, the Server must process the message while ignoring the unknown data (standard Protobuf behavior).

### 6. Tenant Isolation (The Bulkhead)

ArqonBus is a multi-tenant operating system. A failure or attack in Tenant A must mathematically never impact Tenant B.

* **Logical Isolation:** Every NATS Subject, Valkey Key, and Database Row must be prefixed or indexed by `TenantID`.
* **Resource Isolation:** Rate limits must be enforced at the Edge (Shield) before the message consumes Bus resources.
* **Wildcard Restrictions:** Wildcard subscriptions (e.g., `*`) must never cross Tenant boundaries.

### 7. Security by Design

Security is a baseline constraint, not a feature.

* **Zero Trust:** The Gateway does not trust the Client. The Brain does not trust the Gateway. Every boundary requires validation.
* **Fail Closed:** If a security module (like the Wasm Overseer) fails, times out, or crashes, the traffic is **Blocked**. It is never "allowed by default."
* **Secure Defaults:** The system must refuse to start if configured with default passwords or insecure binding addresses (e.g., `0.0.0.0` without TLS).

### 8. Programmable Safety (The Overseer)

We reject hardcoded compliance logic. Safety requirements (PII scrubbing, Code Injection blocking, Sentiment Analysis) vary by tenant and evolve faster than release cycles. Therefore, safety must be dynamic.

* **Wasm Middleware:** Safety policies must be implemented as **WebAssembly (Wasm) Modules** executed at the Shield (Edge) layer. This allows logic updates without recompiling the binary.
* **Fail Closed Mandate:** If a Safety Module times out, crashes, or returns an error, the traffic is **Blocked**. It is never "allowed by default."
* **Bounded Execution:** To prevent Denial of Service via inspection, all safety modules must have strict, non-negotiable limits on CPU usage (gas) and memory consumption.
* **The Air Gap:** Safety logic runs in a sandboxed environment. It must never have direct access to the host file system, network, or internal state of the Shield.

### 9. Message-as-Program (The Capsule Principle)

Messages are not just passive data envelopes. They may carry **Program Capsules** (Seeds + Rules) that enable Code-Mobile architectures.

* **Digital DNA:** The bus is optimized to transport the *Potential* (Seed), not just the *Result*.
* **Execution at the Edge:** Intelligent endpoints (Operators) unpack and execute these seeds using verified interpreters (NVM, Wasm, BowlNet).

### 10. Delta-First Architecture (ITMD)

Computation cost scales with **Change ($\Delta$)**, not total system size.

* **Diff over Snapshot:** Protocols must prefer incremental updates (Deltas) over full state dumps.
* **Causal Integrity:** Changes are propagated causally. If constraints allow, we process only the difference.

### 11. Circuit-First Orchestration

Topology is declarative.

* **Pipelines are Circuits:** Complex flows (NVM → BowlNet → QML) are defined as **Circuits** (DAGs) in configuration, not hardcoded in service logic.
* **Decoupled Routing:** Operators remain oblivious to their upstream/downstream neighbors.
* **Semantic Kernels:** We prefer routing based on **Emergent Kernels** (Stable Attractors) over raw noise. Discovery operators are responsible for distilling kernels.

---

### Tier Ω Scope (Applies to Sections 12–21)

Sections **12–21** govern **Tier Ω (Experimental)** operators and **Realities** only.

Tier 1 (Production) behavior **must** continue to satisfy all Core Principles in:

* **Section II.1–11** (Layering, Statelessness, Protocol Sovereignty, Tenant Isolation, Security, Safety),
* **Section VIII** (Performance & Hot-Path Invariants),
* **Section IX** (Observability & Telemetry Contracts), and
* **Section X** (Data Governance & Retention),

regardless of any Tier Ω configuration. **No Tier Ω behavior may weaken or bypass those invariants.**

---

### 12. Bounded Emergence (Tier Ω Only)

We consciously work only inside the **Engineerable Sub-Space**.

* **Tier Ω Only:** Even in experimental regimes, we serve architectures where we retain control (Cycle length, Basin shaping, safety envelopes).
* **Chaos Ban:** Highly chaotic or poorly characterized regimes are considered *out of scope* for production systems and live only in research sandboxes.
* **Regime Tuning:** Operators may be tuned to the **Edge of Chaos** (Criticality) for maximum adaptivity, provided sufficient **Damping/Collapse** mechanisms exist to prevent runaway.
* **Dual-Flow Dynamics:** Critical circuits must implement both **Generative** (Creation) and **Dissipative** (Pruning) pressures to prevent stagnation or explosion.
* **Core Invariant Link:** All Tier Ω emergence remains subject to Tenant Isolation (II.6), Security & Safety (II.7–8), and Boundedness (VIII.1–2).

### 13. Temporal Sovereignty (Tier Ω Only)

**Time-varying structure** (e.g., adaptive schedules, sequences of matrices) is a first-class control mechanism, equal to static topology.

* **Dynamic Schedules:** We assume control can be restored through well-designed temporal programs, not just static wiring.

### 14. Mathematical Rigor (Algebraic Preference, Tier Ω Only)

* **Solvers over Heuristics:** If a problem can be solved by a matrix operation or algebraic solver (e.g., GF(2)), do not use a neural net or heuristic.
* **Explicit Control:** Controllers must be explicit and observable. Hidden control loops are forbidden.
* **Structured Sampling:** Prefer **Prime-Structured Grids** (e.g., Prime-CF) over random sampling for parameter sweeps and discovery loops.

### 15. The Omega Tier (Risk Classification, Tier Ω Only)

Operators are classified by risk profile:

* **Tier 1 (Production):** Safe, bounded, and critical. Must pass all strict gates.
* **Tier Ω (Experimental):** Permitted to exhibit complex, hard-to-predict behavior but must be **strictly confined** to non-critical clusters. Tier Ω constraints are stricter regarding isolation and looser regarding predictability.

### 16. Diagnostic Segregation (Tier Ω Only)

* **Signals vs Decisions:** Outputs from Ω-Tier operators (Math Fabrics, experimental substrates) are treated as **Diagnostic Signals** (Microscopes), not direct Decision-Makers in hot-path routing loops.

### 17. The 4-Layer Hierarchy (Tier Ω Only)

Complex emergent circuits follow the standard **Substrate → Observer → Controller → Architect** hierarchy.

* **Explicit Roles:** Operators must implicitly or explicitly fulfill one of these roles.
* **Downward Causation:** Higher-layer operators (Architects) can reconfigure lower layers (Substrates), provided the control messages are explicit and audited.
* **Topology as Control:** Architects may use **Dynamic Rewiring** (Topology Change) as a control actuator to shift system operating regimes.
* **Recursive Operators:** Operators may be statefully recursive. They must explicitly declare **Recursion Depth Limits** and **Halt Conditions**.
* **Meta-Optimizers:** We explicitly recognize **Meta-Optimizers** whose output is the *configuration* of other operators. They are subject to strict "Code-Generation" safety gates (same as human-authored code).
* **Genesis Circuits:** We recognize the **Architect-Creator-Agent** loop as a standard pattern for co-evolutionary workloads.

### 18. Probability Engines (NVM/QTR, Tier Ω Only)

The bus supports **Probability Shaping** engines where outputs are distributions, not scalars.

* **Superposition:** Routing and state can be "superposed" (probabilistic) until explicitly measured by a "Collapse" operator.
* **Hybrid Backends:** The bus supports **Quantum/NVM Backends** as first-class compute nodes. Jobs are routed based on `backend_type` capabilities (e.g., `qtr_sim`, `vqe`).

### 19. Temporal Physics (TTC, Tier Ω Only)

* **Temporal Consensus:** For critical Tier Ω coordination, Time may be modeled as a **BFT-validated ledger**, ensuring all operators agree on the sequence of events.
* **Differential Messaging:** The bus supports **Trajectory Shaping** protocols where signals are differential perturbations of a shared temporal fabric.
* **Phased Operation:** Circuits may explicitly declare phases (e.g., `["explore", "consolidate"]`). Control policies must adapt to the active phase.

### 20. The Reality Factory (Tier Ω Only)

The system manages **Realities** (Bundles of Substrate + Laws + Agents) as first-class lifecycle objects.

* **Reality Objects:** A Reality is a managed container for an emergent ecosystem. It must be explicitly defined and governed.
* **Lifecycle States:** Hosted realities must track their lifecycle state (`Draft` → `Running` → `Promoted` → `Archived`), with strict transition gates and observability.

### 21. Strong Emergence Patterns (FRR/Omega, Tier Ω Only)

* **Criticality as Control:** Operators may explicitly tune control parameters (e.g., coupling $k_c$) to keep substrates at the **Edge of Chaos**, maximizing computational capacity.
* **Wave Interference:** Relational fabrics may exhibit interference. Observers must be capable of measuring **Coherence** and **Decoherence** as first-class metrics.
* **Homeostatic Override:** Feedback Controllers must have the authority to **Override** substrate dynamics (e.g., force State Reset) when error thresholds are breached.
* **Internal Models:** Operators should implement the **Actor-Modeler** pattern, maintaining an internal predictive model of their peers to generate self-supervised error signals (solving the Internal Oracle problem).
* **Curiosity Metrics:** "Surprise" (Model Prediction Error) is a valid optimization target for Discovery Operators, driving the exploration of novel regimes.

---

# III. Code Quality & Engineering Standards

### 1. The "Boring Code" Manifesto

We value clarity over cleverness. ArqonBus is critical infrastructure; it must be readable by a junior engineer at 3 AM during an outage.

* **Readability First:** If a "clever" one-liner creates cognitive load, expand it.
* **Explicit over Implicit:** Magic behavior, monkey-patching, and hidden control flow are forbidden.
* **Standard Tooling:** We adhere strictly to community standards (Rust `clippy` on pedantic, Elixir `credo` on strict).

### 2. Asynchronous Boundaries

Given the high-concurrency nature of the system, blocking the event loop or scheduler is a fatal error.

* **I/O Mandate:** All I/O-facing functions (Database, Network, File System) must be clearly marked `async` (Rust) or rely on non-blocking OTP patterns (Elixir).
* **Purity Mandate:** Business logic (State transitions, Protocol parsing, Policy decisions) must remain **Synchronous and Pure** where possible. Side effects must be pushed to the edges of the architecture.
* **Timeout Mandate:** No I/O operation shall exist without a configured timeout. Infinite waits are forbidden.

### 3. Error Handling Philosophy

Errors are data, not exceptions. They must be handled explicitly.

* **Fail Loud (Developer Errors):** Logic errors, invariant violations, and impossible states must panic or crash the process immediately. We do not run with corrupted state.
* **Fail Soft (Runtime Errors):** Network failures and bad user input must be handled gracefully via degradation or rejection.
* **The "Swallow" Ban:** `try/catch`, `unwrap()`, or `Result::ok()` blocks must **never** silently discard errors.
* **Typed Context:** All errors must have a distinct Type (Enum/Struct) and carry Context (TenantID, RequestID). Stringly-typed errors are forbidden.

### 4. Logging & Observability

* **Structured Only:** All logs must be structured (JSON), containing correlation IDs (`trace_id`) to trace requests across Voltron layers.
* **Level Discipline:** `ERROR` means operator intervention is required. `WARN` means handled anomaly. `INFO` is lifecycle. `DEBUG` is payload details (disabled in prod).
* **Security Redaction:** Logs must never contain PII, Auth Tokens, or raw Message Payloads at `INFO` level or above.

### 5. Configuration Discipline

* **Config Over Code:** Operational thresholds (Timeouts, Buffer sizes, Rate limits) must be defined in configuration/env vars, never hardcoded as magic numbers.
* **Validation on Startup:** The application must validate all configuration at boot. If config is invalid, the application must refuse to start (Fail Loud).

### 6. Deterministic State & Protocol Correctness

Distributed systems die when state transitions become ambiguous.

* **State Machine Contracts:** All critical state transitions (Presence, Room State, Circuit Breakers) must be implemented as explicit State Machines. Hidden mutations inside helper functions are forbidden.
* **Phonic ROM (Physical Determinism):** Physical operators must expose deterministic, replayable interfaces. We do not rely on transient physics; we rely on **Recorded Fingerprints** (Phonic ROM) to ensure identical outputs for identical inputs.
* **Multi-Agent Encapsulation:** Operators may internally contain entire Multi-Agent Systems. The bus treats them as opaque, but metadata must declare `multi_agent: true`.
* **Internal Oracles:** "Observer" operators are treated as **Internal Oracles** — trusted sources of truth for the state of a substrate.
* **Topology is Control:** Changing the network shape (rewiring) is a **Control Action**, equivalent to changing a parameter. It must be logged and audited as such.

### 7. Protocol-First Definition

* The `.proto` schema is the source of truth. Manual parsing of binary data is prohibited; all parsers must be code-generated.
* **Command Strictness:** Every Command/Event must have a single authoritative handler. Dynamic dispatch based on untyped maps is prohibited.

### 8. Memory Safety & Resource Guarantees

We operate in a constrained physical reality. Infinite resources do not exist.

* **Allocation Boundaries:** Hot-path structs (Per-Message processing) must have explicit maximum sizes. Unbounded vectors/maps in request handlers are forbidden.
* **Leak Prevention:** In Rust, `Rc`/`Arc` cycles are forbidden without `Weak` references. In Elixir, process heaps must be monitored by the Supervisor.
* **Resource Caps:** Every subsystem must define hard caps for Memory Usage, Open File Descriptors, and Goroutines/Processes to prevent neighbor starvation in multi-tenant environments.

### 9. Concurrency Safety & Ordering

* **Immutable-by-Default:** Shared state must use immutable data structures (Elixir) or synchronized primitives (Rust `Mutex`/`RwLock`). Unprotected mutation is a build failure.
* **Backpressure is Law:** All producers (e.g., NATS Consumers) must implement bounded queues. If the consumer is slow, we **Shed Load** (Drop Message) rather than OOM the server.
* **Ordering Invariants:** Handlers must never assume execution ordering unless enforced by a causal identifier (Sequence ID). We do not rely on "Scheduler Luck."

### 10. Performance Discipline

* **Hot Path Hygiene:** No heap allocations allowed inside the hot message loop. No string formatting or dynamic dispatch in the routing path.
* **Latency Budgets:** All APIs must define a max tolerable latency (e.g., "Heartbeat < 10ms p99"). Operations exceeding this budget must yield or fail.

### 11. API & Interface Stability

* **Boundary Contracts:** Internal modules must communicate via stable `Trait` (Rust) or `Behaviour` (Elixir) interfaces, never raw structs.
* **Flag Hygiene:** Feature Flags are allowed at the Edge (Shield) for rollout control but forbidden inside the core Routing Loop to prevent combinatorial logic explosions.

### 12. Dependency Hygiene

* **Admission Rules:** New dependencies are guilty until proven innocent. They must be actively maintained and necessary.
* **Pinning:** All dependencies must be version-pinned. Wildcards (`*`) are forbidden.

### 13. Documentation Standards

* **Docs as Code:** Documentation must live in the repo. Any PR that changes behavior must update the corresponding `doc/`.
* **Decision Records:** Architectural decisions (ADRs) must be committed to the repo to explain **why** a decision was made.

### 14. Build & Artifact Integrity

* **Reproducibility:** Builds must be deterministic. The same commit hash must produce the exact same binary hash, bit-for-bit.
* **Binary Hygiene:** Debug symbols and unused features must be stripped from Production binaries.
* **Security Flags:** Rust binaries must compile with Stack Protection, LTO, and max optimization levels (`-C opt-level=3`).

### 15. Mathematical Rigor (Algebraic Preference)

* **Solvers over Heuristics:** If a problem can be solved by a matrix operation or algebraic solver (e.g., GF(2)), do not use a neural net or heuristic.
* **Explicit Control:** Controllers must be explicit and observable. Hidden control loops are forbidden.
* **Structured Sampling:** Prefer **Prime-Structured Grids** (e.g., Prime-CF) over random sampling for parameter sweeps and discovery loops.

---

# IV. Testing Strategy & Quality Gates

### 1. TDD as the Working Standard

Test-Driven Development (TDD) is the **default and expected workflow** for all ArqonBus components.

* **The Workflow:**
  1. **Specify:** Define behavior in `/specs/` (SDD-first).
  2. **Test:** Write or extend tests that express that behavior.
  3. **Implement:** Write the code that satisfies the tests.
  4. **Refactor:** Optimize while keeping the suite green.
* **The Refactoring Standard:** If strict TDD is impractical for a refactor, tests must **already exist** asserting the behavior. Any change to observable behavior requires a Spec Update.
* **The Safety Standard (CASIL):** The Overseer (Wasm Safety Layer) operates under a **Strict TDD Regime**. No Safety code merges without tests covering:
  * Classification Logic.
  * Boundary Failures (Timeouts, OOM).
  * Abuse Cases (Malicious Payloads).
  * Fail-Closed verification.

### 2. Coverage Expectations (Per Subsystem)

Coverage is about behavioral exhaustiveness, not raw percentages.

* **Routing Core:** Must cover empty rooms, race conditions, reconnection storms, backpressure triggering, and tenant-boundary enforcement.
* **Command Handlers:** Must have **Unit Tests** (Logic/Parsing) and **Integration Tests** (End-to-End via Shield/Spine/Brain).
* **History & Persistence:** Must assert deterministic ordering, retention contract adherence, and replay semantics.
* **Envelope Validation:** Must include Valid, Invalid, and Adversarial payloads (Fuzzed bytes, Schema mismatches).
* **Telemetry (OFD):** Must verify metric cardinality, log structure (JSON), and Trace ID propagation.
* **The Overseer:** Must simulate all policy actions (`ALLOW`/`REDACT`/`BLOCK`), resource limits, and multi-tenant policy interactions.
* **Battery Evaluation:** Complex Tier-Ω operators must be validated against a **Multi-Task Battery** to prove generality and stability before production.

### 3. Test Discipline Requirements

* **Unit Tests:** Must run in milliseconds with **Zero** external dependencies.
* **Integration Tests:** Must run via Docker Compose (simulating NATS/Postgres/Valkey) and simulate real clients.
* **Flaky Tests:** Flaky tests are **Critical Bugs**. A PR that reveals flakiness must resolve the flake before merge.
* **Adversarial Simulation:** Every subsystem must include tests that verify stability under chaos (panics, partition, flood) and adherence to Zero Trust defaults.
* **Determinism:** Tests must avoid random sleeps and time-dependent logic. Use controlled clocks.

### 4. Quality Gates

A PR **may not be merged** if any of the following are true:

* **Protocol Gate:** Public behavior changes without tests, or new Fields without `.proto`/Spec updates.
* **Coverage Gate:** Coverage decreases in critical subsystems or Integration tests are missing for cross-service logic.
* **Architecture Gate:** Shield contains business logic, Brain handles raw sockets, or any layer bypasses the Spine.
* **Safety Gate:** Safety/Policy changes without matching tests, documentation, and "Fail-Closed" verification.
* **Observability Gate:** New handlers emit no metrics, or logs lack Correlation IDs.
* **Technical Debt Gate:** New `TODO`s added without a tracked `TD-xxx` ID and TTL.
* **Spec Gate:** Behavior implemented without a Spec, or Spec not updated to match Code.

### 5. Automated Enforcement (The ACES Pipeline)

The ACES CI pipeline enforces this Constitution as hard rules.

* **Mechanical Enforcement (Conftest):** Policy-as-code checks for architecture invariants, coverage thresholds, and technical debt TTLs.
* **Semantic Enforcement (Sentinel Bot):** Automated review that flags missing Spec updates, architectural violations, and missing observability fields.
* **Execution Enforcement (CI Runners):** Blocks merges on failing gates, runs full integration suites, executes chaos tests, and verifies performance regression baselines.

> **Summary:** Every behavior must be defined by a Spec, enforced by a Test, and validated by the Pipeline. There are no "Admin Merges."

---

# V. Lifecycle & Automation

ArqonBus does not “ship code”; it manufactures **artifacts** through a controlled factory. Anything outside the factory is undefined behavior.

### 1. The Factory Mandate

Manual releases are strictly forbidden. The CI/CD pipeline is the **only** path to production.

* **The Pipeline is Sovereign:** If it did not pass CI, it does not exist. SSHing into production to “hotfix” code is prohibited.
* **No Snowflake Hosts:** All environments (dev, staging, production) must be provisioned from code (Terraform, Ansible, etc.), never hand-crafted.
* **Single Source of Truth:** Deployment manifests (Kubernetes, Nomad, etc.) live in version control and are the only authoritative description of what runs in production.

### 2. Immutable & Reproducible Artifacts

* **Immutable Artifacts:** Every release must produce immutable Docker images and binaries identified by a content hash (e.g., SHA-256). Re-tagging an image (e.g., moving the `:latest` tag to a different hash) is forbidden in production manifests.
* **Reproducible Builds:** The build system must be deterministic. Building the same commit hash on two different machines must yield bit-for-bit identical binaries.
* **No Local Builds to Prod:** Production artifacts are built only in CI from a clean environment. Developer laptops are not part of the trusted build chain.

### 3. Supply Chain Security

* **SBOM Mandate:** Every release artifact must generate a Software Bill of Materials (SBOM) that lists all dependencies.
* **Signing:** All binaries and container images must be cryptographically signed. Unsigned artifacts may not be deployed.
* **Dependency Locking:** No floating versions. `Cargo.lock` and `mix.lock` must be committed. Dependencies must be pinned and audited automatically; high-severity vulnerabilities block the release.
* **Provenance:** The origin of each artifact (commit, branch, CI run) must be traceable from production back to source.

### 4. Semantic Versioning & Compatibility

We adhere strictly to Semantic Versioning (SemVer 2.0.0) for both **Protocol** and **Public API**, consistent with Section II.5.

* **The Contract:** Version numbers communicate compatibility, not marketing.
  * **MAJOR:** Breaking protocol or API changes (see II.5).
  * **MINOR:** New, backward-compatible features (additive only).
  * **PATCH:** Bug fixes and internal improvements with no behavioral change.
* **Deprecation Policy:** No field, command, or feature may be removed in a Major version without being marked `deprecated` in at least one previous Minor version.
* **Rolling Upgrade Guarantee:** The system must support at least **N-1** compatibility. A `v1.2` Gateway must be able to communicate with a `v1.1` Brain during a rolling deployment, and vice versa where documented.

### 5. Deployment Safety Nets

Deployments must be **boring, reversible, and observable.**

* **Canary First:** New releases must be rolled out to a small slice of traffic or tenants first. If error rates or SLOs degrade for the canary, the rollout must halt automatically.
* **Shadow Traffic:** When changing core routing or safety behavior, the new version must be tested using mirrored (shadow) traffic before it is allowed to affect real users.
* **Rollback as First-Class:** Every release must have a tested, documented, single-command rollback procedure. If rollback is not safe and tested, the release is not allowed.
* **No Big Bang:** Global, all-at-once deploys of core components (Shield, Spine, Brain, Storage) are forbidden.

---

# VI. Operational Excellence

ArqonBus is critical infrastructure. The way it behaves in production is as important as the way it behaves in tests.

### 1. Service Level Objectives (SLOs) & Error Budgets

* **Defined SLOs:** Every externally visible surface (connect, publish, subscribe, presence, history) must have explicit SLOs (e.g., latency percentiles, availability, delivery success).
* **Error Budgets:** Each SLO must define an error budget. When a budget is exhausted, feature development for that surface halts until reliability is restored.
* **SLO-Driven Decisions:** SLOs and error budgets drive release pace and prioritization. They are not vanity metrics.

### 2. Incident Response

* **Blameless Postmortems:** Every SEV-1 (Critical) incident requires a written postmortem focused on systemic causes, not individual blame.
* **Root Cause is a System Property:** We do not accept “human error” as the terminal explanation. We ask why the system allowed the error, and how to make that class of error impossible or less likely.
* **Incident Timelines:** Major incidents must produce a clear timeline: detection, diagnosis, mitigation, remediation.
* **Runbook Mandate:** Authorized operational actions must be documented as `runbook.md` entries. Acting outside a runbook during an incident is allowed only when explicitly declared a “Novel Condition” and must be captured in the postmortem.

### 3. On-Call & The Pager

* **Rotation:** Core subsystems (Shield, Spine, Brain, Storage, Overseer) must each have an on-call owner rotation.
* **Actionable Alerts Only:** Alerts must be tied to SLOs and must be actionable. Chronic noisy alerts are an operational bug and must be fixed.
* **Follow-the-Sun Ready:** The system must be operable by someone who is not the original author, using only logs, metrics, traces, and runbooks.

### 4. Data Stewardship & Retention

* **Retention is Explicit:** No infinite retention by accident. Every stream, log, and table must have either:
  * A defined retention period (TTL), or
  * A documented archival strategy.
* **Tenant Data Isolation:** Tenant data (configuration, history, logs) must be logically isolated and addressable by `TenantID`, and any cross-tenant access must be impossible without explicit policy.
* **Right to Vanish:** When a tenant is deleted or requests erasure (where applicable), their data must be deleted or cryptographically shredded across:
  * Primary storage
  * Secondary indexes
  * Caches
  * Logs and backups (subject to regulatory allowances and obligations)
* **Privacy by Design:** Data used for diagnostics or training (if any) must be anonymized and aggregated.

### 5. Performance & Capacity Invariants

Performance is not a “nice to have”; it is a contract.

* **Bounded Queues:** No unbounded queues in hot paths. Backpressure or shedding is mandatory.
* **Graceful Degradation:** Under overload, the system must prefer:
  1. Shedding non-critical traffic
  2. Reducing features
  3. Limiting concurrency  
     rather than failing catastrophically.
* **Per-Tenant Guardrails:** Resource usage (connections, subscriptions, history volume) must be bounded per tenant to prevent “noisy neighbor” failures.
* **Latency Budgets:** Core paths (connect, publish, presence updates) must have documented latency budgets. Exceeding budgets is an SLO violation, not a suggestion.
* **Fractal Awareness:** Controllers must assume **Long-Range Correlations** (Fractal Time) in load and error metrics, rather than assuming white noise.

### 6. Observability & Audit

Operational correctness depends on the ability to observe and reconstruct events.

* **End-to-End Tracing:** Requests flowing through Shield → Spine → Brain → Storage must be traceable via correlation IDs. Missing IDs in hot paths are considered bugs.
* **Structured Telemetry:** Logs, metrics, and traces are required for:
  * All error paths
  * All state changes that affect routing, presence, or safety
* **Security Audit Trails:** Authentication and authorization decisions must be logged (without secrets) with enough context to support forensics.
* **No Silent Failures:** Any automatic recovery or fallback must emit telemetry that can be used to analyze frequency and impact.
* **Standard Observatories (Tier Ω):** Every Reality that hosts Tier Ω operators must have attached **Observatories** (Safety, Physics, Population) to be considered valid.

### 7. Collapse Monitoring (Liveness)

Emergent systems tend to "die" (collapse into fixed points).

* **Liveness Metrics:** Emergent operators must emit liveness metrics (e.g., Variance over Time, Entropy).
* **Dead Zone Pruning:** Permanently static ("dead") zones must be automatically deprioritized or pruned to save resources.

### 8. Semantic Depth & Curiosity

* **Twist/Overlay Complexity:** We monitor "Semantic Depth" (how many interpretive layers or "twists" a message passes through). Circuits exceeding depth thresholds require governance review.
* **Curiosity Metrics:** Operators may emit "Interestingness" or "Novelty" scores. These are valid signals for routing/control (Exploration vs Exploitation).
* **Spectral Blueprints:** Operators must expose **Spectral Signatures** (Resonance profiles) to allow automated drift detection against their design blueprint.
* **Overlay Density:** We monitor **Relational Density** (count of overlapping dependencies/factors) as a proxy for structural fragility.
* **Overlap Topology:** We measure system coupling via **Geometric Overlap** (Prime Intersections). High-overlap zones are treated as high-contention critical sections.

### 9. Artifact Provenance

Configuration artifacts generated by **Discovery Operators** (Tier Ω) are treated as code.

* **Provenance Tagging:** Artifacts must be tagged with Source, Version, and Parent IDs before admission to production.
* **Review Gates:** Automated artifacts must pass the same policy gates as human-authored config.
* **Emergent Schemas:** We support **Emergent Schemas** where structured events (Particles/Kernels) are distilled from raw substrates by Discovery Operators.
* **Code-Bearing Config:** Configuration artifacts may contain **Evolved Code** (Wasm/AST). These are subject to the same CI/Safety gates as human code.
* **Promotion Gates:** Artifacts generated in Sandbox Realities must pass **Observatory Gates** before being promoted to Production.

---

# VII. Governance & Amendment

Governance defines how ArqonBus protects its mission and how this Constitution itself may change.

### 1. Scope Protection

ArqonBus is **Transport and Coordination Infrastructure**, not a general-purpose application runtime.

* **Application Runtime Ban:** Proposals to run arbitrary application logic on the core bus (e.g., long-running workflows, generic compute runtimes inside the Shield/Brain) must be rejected. Application code belongs in external workers.
* **Feature Creep Valve:** Features that do not directly serve the mission of “Coordination Fabric for Humans, Devices, and Intelligences” must be implemented as:
  * External services, or
  * Optional plugins, not core behavior.

### 2. Complexity Budget

Complexity is technical debt with compound interest.

* **Justification of Complexity:** Adding a heavy dependency (new database, language runtime, broker, or orchestration system) requires a written justification and Architecture Review approval.
* **One-Way Rule:** For any given concern (authentication, configuration, transport), there should be one obvious way. Competing mechanisms for the same purpose are prohibited unless explicitly justified as transitional.
* **ADR Requirement:** Major architectural decisions must be recorded as Architecture Decision Records (ADRs) in the repo. “We forgot why we did this” is not acceptable.

### 3. Decision Making

* **Architecture Review Board (ARB):** A small set of core maintainers is responsible for enforcing this Constitution and approving major architectural changes.
* **Tiebreaker:** In the presence of multiple viable designs, the ARB prefers:
  * Simpler over more complex
  * Observable over opaque
  * Explicit over implicit
  * Proven patterns over novelty
* **Doctrine Hierarchy:**
  1. The Constitution (this document)
  2. Engineering Doctrine (e.g., SOTA Engineering Doctrine)
  3. Policies & Playbooks (CI, Sentinel, Policy-as-Code rules, runbooks)
  4. Implementation details  

  Lower layers must not contradict higher layers. When in doubt, the Constitution wins.

### 4. Amendments

This Constitution is living but intentionally hard to change.

* **Ratification:** Amendments to this document require a unanimous vote by all core maintainers.
* **Change Driver:** Amendments must be justified by:
  * Production learnings (incidents, SLO data), or
  * Demonstrable architectural constraints in new epochs (e.g., Multi-Agent Systems), not purely theoretical preferences.
* **Traceability:** Every amendment must link to:
  * An ADR
  * Relevant incidents or metrics
  * The discussion that led to the change

### 5. Interpretation

* **Constitution Over Convenience:** “It was easier this way” is never a valid reason for violating this document.
* **Guardian Tools:** Sentinel, ACES, and Policy-as-Code are empowered to block changes that violate this Constitution. Their behavior should be tuned **toward** stricter conformity over time, not looser.
* **Last Word:** If any ambiguity remains, the ARB interprets the Constitution, documented via ADR, and tooling is updated to reflect that interpretation.

---

# VIII. Performance & Hot-Path Invariants

ArqonBus is real-time infrastructure. Performance is not an optimization; it is a **correctness property**.  
The system must behave predictably under load, degradation, and failure.

### 1. Boundedness as Law

Unbounded anything is a denial-of-service vector.

* **No Unbounded Queues:** All queues in hot paths (Shield, Spine, Brain) must be bounded. When limits are reached, the system must:
  1. Apply backpressure,
  2. Shed load, or
  3. Fail fast with a clear error,  
     but must never silently buffer until OOM.
* **Memory Caps:** Per-connection, per-tenant, and per-node memory consumption must have hard upper bounds. Exceeding a bound triggers shedding or disconnection, not uncontrolled growth.
* **CPU Boundaries:** CPU-intensive tasks (compression, encryption, large JSON serialization, ML inference) must not run in the main reactor or scheduler loops. They must be offloaded to dedicated pools with quotas.

### 2. Hot-Path Constraints

The hot path is any operation that runs per-message, per-connection, or per-heartbeat.

* **No Blocking in Hot Paths:** Synchronous disk I/O, synchronous network calls, and blocking locks are forbidden in hot paths.
* **O(1) or Amortized O(1):** Per-message operations in the Shield and Spine must be O(1) or amortized O(1) with respect to number of tenants, rooms, or subscribers.
* **Constant-Time Tenant Identification:** Tenant routing and authorization decisions must be made in constant time relative to total tenant count. Linear scans over tenants or large global maps in hot paths are forbidden.
* **No Dynamic Allocation in Tight Loops:** Hot loops must avoid unnecessary heap allocations, string formatting, and dynamic dispatch. Where allocations are required, pooling or arena strategies must be used.

### 3. Throughput & Latency Behavior

Exact numerical SLOs are defined elsewhere; this section defines **behavioral invariants**.

* **Deterministic Under Load:** Under load, the system must degrade in a controlled, monotonic manner. It must not exhibit oscillatory or chaotic behavior (e.g., thrashing, repeated recovery loops).
* **Locality of Failure:** Overload in Tenant A must never cause global failure for Tenant B. Degradation must be tenant-local wherever mathematically possible.
* **Latency Budgets:** Every core operation (connect, subscribe, publish, presence update) must have a defined latency budget. Code that consistently violates its budget is considered incorrect, even if it “works.”
* **No Priority Inversion:** High-priority paths (e.g., heartbeats, control messages) must not be starved behind lower-priority work. Queueing and scheduling must reflect this.

### 4. Capacity & Scaling Invariants

* **Horizontal First:** When capacity is exceeded, the default solution is to scale horizontally using stateless Shield nodes and partitioned Spine/Brain/Storage roles, not to push vertical complexity into single nodes.
* **Predictable Scaling:** Scaling characteristics (connections per node, messages per second per node) must be measured and documented. Surprises in scaling behavior are treated as architectural bugs.
* **No “Special” Nodes:** There must be no single irreplaceable node. Roles may be specialized (e.g., leader), but any node must be replaceable through automation.

---

# IX. Observability & Telemetry Contracts

What cannot be observed cannot be governed. Observability is a **first-class protocol** of ArqonBus, not a bolt-on feature.

### 1. Logs, Metrics, Traces as First-Class Citizens

* **Structured Logs Only:** All logs must be structured (JSON or key/value) and must include correlation identifiers (`trace_id`, `span_id`, `tenant_id`, `node_id`) wherever applicable.
* **Metrics Are Mandatory:** Any new externally visible behavior (API, command, state transition, safety decision) must emit metrics. Absence of metrics for operator-relevant behavior is a constitutional violation.
* **Tracing is Mandatory for Hot Paths:** All hot paths (connect, routing, presence, safety) must be wrapped in spans, with propagation across Shield → Spine → Brain → Storage.

### 2. Telemetry Contracts

Telemetry itself is an interface and must be treated as such.

* **Versioned Telemetry:** Metric names, labels, and log schemas that operators depend on must be versioned and evolved carefully. Breaking telemetry changes must be documented and rolled out like API changes.
* **Cardinality Discipline:** Metrics must not use unbounded high-cardinality labels (e.g., user IDs, raw IPs, arbitrary strings). Cardinality explosions in metrics are considered operational bugs.
* **Stability:** Once introduced, telemetry must remain stable enough to support dashboards, alerts, and SLO calculations. Renaming or removing must go through review.

### 3. Security & Privacy in Telemetry

* **No Secrets in Telemetry:** Logs, metrics, and traces must never contain secrets, tokens, passwords, or sensitive payloads. Redaction must be applied at the source.
* **Minimal PII:** Personally identifying information must be minimized and, where present, must comply with privacy commitments (hashing, anonymization, or opt-in flagging).

### 4. Operability Guarantees

* **Debuggability:** For any production issue impacting tenants, the combination of logs, metrics, and traces must allow an on-call engineer (who is not the original author) to reconstruct the key events without guesswork.
* **No Silent Recovery:** Automatic retries, fallbacks, or recoveries must emit telemetry with enough information to quantify frequency and cost.
* **Alertability:** Observability data must support actionable alerts for:
  * SLO violations
  * Safety module failures
  * Tenant isolation breaches (attempted or actual)
  * Capacity and backpressure thresholds

---

# X. Data Governance & Retention

ArqonBus handles tenant data, configuration, and message history. Data is an asset and a liability. Its lifecycle must be explicit.

### 1. Data Classification

All data handled by ArqonBus must be classified into categories, with clear rules:

* **Configuration Data:** Tenant configuration, routing rules, safety policies.
* **Operational Data:** Metrics, logs, traces.
* **Message History:** Streams of messages (for time travel, replay, and auditing).
* **Security Data:** Access logs, auth decisions, safety decisions.

Each category must define:

* Retention policy
* Access control model
* Encryption requirements
* Backup and restore procedures

### 2. Retention & Time Travel

* **Explicit Retention:** No data may be stored without an explicit retention policy (TTL) or archival strategy. “Keep forever” is allowed only with explicit legal/compliance justification.
* **Time Travel Bounds:** Replay and history features must operate within configured and documented time windows. Beyond that window, data may not be assumed present.
* **Archival vs. Online:** Hot storage and cold/archive storage must be clearly separated in design and configuration. Accessing archival data must not impact hot-path latency.

### 3. Tenant Isolation & Privacy

* **Tenant-Scoped Data:** All tenant data must be addressable by `TenantID` and isolated logically and physically where required. Queries or operations that span tenants are forbidden unless explicitly allowed at a governance level (e.g., global ops-only diagnostics).
* **Right to Vanish:** When a tenant is deleted or requests erasure (where applicable), their data must be deleted or cryptographically shredded across:
  * Primary databases
  * Search indexes
  * Caches
  * Logs and backups (subject to regulatory allowances and obligations)
* **Access Controls:** Internal access to tenant data (for support or diagnostics) must be:
  * Authenticated and authorized
  * Audited with immutable logs
  * Restricted to least privilege roles

### 4. Encryption & Key Management

* **Encryption in Transit:** All external client traffic must use TLS or equivalent secure transport.
* **Encryption at Rest:** Sensitive data (config, history, security logs) must be encrypted at rest with managed keys.
* **Key Lifecycle:** Keys must be rotated, revoked, and audited. Hardcoding keys or long-lived static secrets is forbidden.

---

# XI. Internal Service Contracts & Complexity Escalation

The internal structure of ArqonBus must remain understandable, evolvable, and safe. Service contracts and complexity controls make this possible.

### 1. Internal Service Contracts

Communication between Voltron layers and internal services must be formal and stable.

* **Versioned Contracts:** All interactions between Shield, Spine, Brain, and Storage must use versioned, explicit contracts (Protobuf schemas, RPC interfaces, NATS subjects). Ad-hoc “just send JSON” between services is forbidden.
* **Backward Compatibility:** Internal contracts must follow the same compatibility rules as external ones:
  * Additive changes are preferred.
  * Breaking changes must be deliberate, versioned, and coordinated with rolling upgrades.
* **No Hidden Side Channels:** “Backdoor” channels (e.g., direct DB reads, hidden HTTP endpoints) that bypass formal contracts are prohibited.
* **Contract as Law:** If behavior is not expressed in a contract (schema + spec), it is not considered a supported path and must not be relied upon.

### 2. Complexity Budget & Escalation

Complexity is treated as a consumable budget, not an accident.

* **Trigger Conditions:** The following always require a Design Review Document (DRD) and Architecture Review:
  * Introducing a new core dependency (database, message broker, major framework).
  * Introducing a new language/runtime into production services.
  * Changing the failure domain (e.g., global coordination, cross-region consensus).
  * Adding a second mechanism for something that already has a working solution (e.g., a second auth system).
* **Design Review Document (DRD):** Must include:
  * Problem definition and context
  * Proposed design and alternatives
  * Invariants and failure modes
  * Impact on SLOs, security, and multi-tenancy
  * Migration and rollback plan
* **ADR Requirement:** Accepted designs must be recorded as Architecture Decision Records (ADRs) committed to the repo. ADRs are the historical memory of the system.

### 3. Golden Paths & De-duplication

* **One Obvious Way:** For any core concern (auth, config, routing, safety evaluation), there should be one preferred “golden path.” Competing mechanisms require explicit, time-bounded justification.
* **Sunsetting Complexity:** When multiple mechanisms exist (during migration), they must have:
  * A defined sunset date
  * A migration plan
  * Telemetry to track use of the legacy path
* **No Eternal Experiments:** Experimental flags or “temporary” code paths must either graduate to supported status (with contracts, specs, tests) or be removed.

### 4. Governance Integration

* **Constitution over Convenience:** Complexity must never be justified solely by convenience in the short term.
* **Sentinel & Policy-as-Code:** Internal contracts and complexity triggers must be enforced by Sentinel and policy-as-code where possible (e.g., detecting new deps, new languages, forbidden imports).
* **ARB Oversight:** The Architecture Review Board has final authority on whether proposed complexity aligns with the Constitution and long-term health of ArqonBus.

---

# XII. Glossary & Canonical Definitions

To prevent interpretation drift (especially for Spec Kit agents), we define core vocabulary used throughout this Constitution.

| Term | Definition |
| :--- | :--- |
| **ArqonBus** | The real-time **Transport and Coordination Infrastructure** described by this Constitution. Not an application runtime or primary data store. |
| **Agent** | An Operator capable of autonomous goal-seeking behavior and state retention (may be software, human-assisted, or hybrid). |
| **Operator** | A generalized actor (Code, Human, or Machine) connected to the Bus that sends, receives, or transforms messages. |
| **Shield** | The edge gateway service (Rust) handling connections, protocol normalization, rate limits, and safety (Overseer). Holds zero business state. |
| **Spine** | The internal message bus (e.g., NATS/Rust) providing transport and decoupling. All internal traffic traverses the Spine. |
| **Brain** | The control plane (Elixir) responsible for complex state, presence, authorization, and system self-healing. |
| **Storage** | The set of services providing durable configuration and hot counters (e.g., Postgres + Valkey) under strict state locality rules. |
| **Capsule** | A message containing a "Seed" (Code/Intent) + "Rules" for execution (Message-as-Program). |
| **Reality** | A governed namespace or container that bundles Substrate + Laws + Agents, particularly for Tier Ω emergent workloads. |
| **Tier 1** | Production-critical operators and circuits. Subject to **all** Core Principles (II.1–11, VIII–X) with strict safety and performance guarantees. |
| **Tier Ω** | Experimental operators and Realities allowed to explore complex/emergent behaviors, but **only** within the constraints and isolation rules of Sections II.12–21, VI.7–9, and VIII–X. |
| **Emergence** | Complex, macro-level behavior arising from many interacting operators. Governed by Bounded Emergence and Tier Ω policies. |
| **Voltron Pattern** | The strict 4-layer architectural hierarchy: Shield → Spine → Brain → Storage. No bypasses allowed. |
| **Universe** | The collection of all Realities and Tenants operating on ArqonBus under this Constitution. |
| **Drift** | Divergence of system behavior or schema from its baseline over time; controlled via SLOs, observability, and Tier Ω governance. |

---

**Verified as Constitution v2.1 (Spec Kit–Aligned)**

