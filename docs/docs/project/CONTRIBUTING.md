# Contributing to ArqonHPO

<div align="center">

**You're about to contribute to the future of optimization infrastructure.**

Every line of code you write here will help engineers around the world make their systems faster, safer, and smarter.

**Welcome to something meaningful**

</div>

---

## 💡 Why Contribute?

### Your Impact

| What You Build     | Who You Help                                                |
| ------------------ | ----------------------------------------------------------- |
| A faster algorithm | The ML engineer waiting 3 days for HPO to finish            |
| A clearer doc page | The new developer at 2am trying to tune their first model   |
| A bug fix          | The SRE whose production system just learned to heal itself |
| A test case        | Every future contributor who won't break what you protected |

### What You Get

- 🏆 **Recognition** — Contributors credited in releases and CHANGELOG
- 🎓 **Learning** — Work with cutting-edge Rust, optimization algorithms, and systems design
- 🤝 **Community** — Join a team that actually reviews PRs thoroughly and kindly
- 📈 **Portfolio** — Your name on production-grade, battle-tested code

---

## 🚀 Quick Start

### 5 Minutes to Your First Contribution

```bash
# 1. Fork on GitHub, then:
git clone https://github.com/YOUR_USERNAME/ArqonHPO.git
cd ArqonHPO

# 2. Build everything
just build

# 3. Run tests (they should all pass)
just test

# 4. Make your change, then
just check  # Format + lint
just test   # Verify nothing broke

# 5. Push and open a PR!
```

### Don't Have `just`?

```bash
cargo install just
# or on macOS: brew install just
```

---

## 🎯 Where to Start

### Good First Issues

We specifically label issues for new contributors:

| Label              | Meaning                    |
| ------------------ | -------------------------- |
| `good first issue` | Perfect for your first PR  |
| `help wanted`      | We actively want help here |
| `docs`             | Documentation improvements |
| `tests`            | Test coverage gaps         |

**[Browse Good First Issues →](https://github.com/novelbytelabs/ArqonHPO/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)**

### Contribution Ideas by Skill

<details>
<summary><strong>🦀 Rust Developer</strong></summary>

<li>Implement new optimization strategies (CMA-ES, SMAC)</li>
<li>Optimize hot path performance (we love benchmarks!)</li>
<li>Add GPU acceleration (CUDA/Metal)</li>
<li>Improve error handling and edge cases</li>

</details>

<details>
<summary><strong>🐍 Python Developer</strong></summary>

<li>Expand Python bindings with new features</li>
<li>Add integration examples (PyTorch, JAX, etc.)</li>
<li>Write cookbook recipes</li>
<li>Improve type hints and docstrings</li>

</details>

<details>
<summary><strong>📝 Technical Writer</strong></summary>

<li>Improve existing documentation clarity</li>
<li>Add more code examples</li>
<li>Create tutorials and guides</li>
<li>Translate docs to other languages</li>

</details>

<details>
<summary><strong>🧪 QA/Testing</strong></summary>

<li>Add edge case tests</li>
<li>Improve test coverage</li>
<li>Property-based testing with `proptest`</li>
<li>Fuzz testing critical paths</li>

</details>

<details>
<summary><strong>🎨 Designer</strong></summary>

<li>Improve TUI layout and UX</li>
<li>Dashboard UI enhancements</li>
<li>Documentation diagrams</li>
<li>Logo and branding</li>

</details>

---

## 📋 Development Workflow

### 1. Create a Branch

```bash
git checkout -b type/short-description
# Examples:
# feat/multi-objective-optimization
# fix/tpe-bandwidth-edge-case
# docs/kubernetes-guide
```

### 2. Make Your Changes

Write code. Write tests. Write docs. In that order.

### 3. Run the Quality Gauntlet

```bash
# The full check (what CI runs)
just check       # Format (rustfmt, ruff)
just lint        # Lint (clippy, ruff, mypy)
just test        # All tests
just coverage    # Coverage report

# Quick iteration
just quick       # Fast format + test
```

### 4. Commit with Meaning

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**

| Type       | Use For                  |
| ---------- | ------------------------ |
| `feat`     | New features             |
| `fix`      | Bug fixes                |
| `docs`     | Documentation            |
| `test`     | Test additions           |
| `refactor` | Code refactoring         |
| `perf`     | Performance improvements |
| `ci`       | CI/CD changes            |
| `chore`    | Maintenance              |

**Examples:**

```bash
feat(core): add CMA-ES strategy

Implements the Covariance Matrix Adaptation Evolution Strategy
for high-dimensional continuous optimization.

- Adds CmaEs struct with standard parameters
- Integrates with existing Strategy trait
- Includes 15 unit tests

Closes #42
```

```bash
fix(bindings): handle empty parameter bounds

The Python bindings would panic when given an empty bounds dict.
Now returns a clear ValueError with guidance.

Fixes #123
```

### 5. Open a Pull Request

- Fill out the PR template completely
- Link related issues
- Add screenshots/recordings for UI changes
- Request review from maintainers

---

## 🏗️ Architecture Overview

Understanding the codebase:

```
ArqonHPO/
├── crates/
│   ├── core/           # 🧠 Solver, strategies, PCR algorithm
│   │   ├── machine.rs      # Main state machine
│   │   ├── strategies/     # NM, TPE, Multi-start
│   │   ├── classify.rs     # Landscape classifier
│   │   └── probe.rs        # LDS sampling
│   │
│   ├── hotpath/        # ⚡ Safety-critical real-time code
│   │   ├── executor.rs     # SafetyExecutor
│   │   ├── spsa.rs         # SPSA optimizer
│   │   └── telemetry.rs    # Ring buffers
│   │
│   └── cli/            # 🖥️ Command-line interface
│       ├── main.rs         # Entry point
│       ├── tui.rs          # Terminal UI
│       └── dashboard.rs    # Web UI
│
├── bindings/
│   └── python/         # 🐍 PyO3 bindings
│
└── docs/               # 📚 MkDocs documentation
```

### Key Concepts

| Concept                  | File          | What It Does               |
| ------------------------ | ------------- | -------------------------- |
| PCR Pipeline             | `machine.rs`  | Probe→Classify→Refine flow |
| Safety Executor          | `executor.rs` | Guardrails + rollback      |
| LDS Sampling             | `probe.rs`    | Low-discrepancy sequences  |
| Landscape Classification | `classify.rs` | Structured vs chaotic      |

---

## 📏 Code Standards

### Rust

```rust
// ✅ Good: Clear, documented, tested
/// Computes the adaptive learning rate for iteration k.
///
/// Uses the SPSA decay schedule: a_k = a₀ / (k + 1 + A)^α
///
/// # Arguments
/// * `k` - Current iteration (0-indexed)
///
/// # Returns
/// Learning rate for this iteration
pub fn learning_rate(&self, k: u64) -> f64 {
    self.initial_rate / (k as f64 + 1.0 + self.stability).powf(self.alpha)
}

// ❌ Bad: Magic numbers, no docs
pub fn lr(&self, k: u64) -> f64 {
    self.a / (k as f64 + 11.0).powf(0.602)
}
```

### Python

```python
# ✅ Good: Typed, docstring, examples
def sample_at(self, index: int) -> dict[str, float]:
    """Generate a deterministic sample at the given index.

    Args:
        index: Global LDS index (0 to 2^64-1)

    Returns:
        Parameter dict with values in bounds

    Example:
        >>> probe = ArqonProbe(config)
        >>> probe.sample_at(0)
        {'x': 0.5, 'y': 0.5}
    """
    ...

# ❌ Bad: No types, no docs
def sample(self, i):
    return self._inner.sample(i)
```

### Tests

Every PR should include tests. We aim for:

- **Unit tests** for individual functions
- **Integration tests** for module interactions
- **Property tests** for invariants (use `proptest`)
- **Edge cases** explicitly tested

```rust
#[test]
fn test_learning_rate_decays_monotonically() {
    let spsa = Spsa::new(42, 3, 0.1, 0.01, SpsaConfig::default());

    let rates: Vec<f64> = (0..100).map(|k| spsa.learning_rate(k)).collect();

    for window in rates.windows(2) {
        assert!(window[0] > window[1], "Learning rate should decay");
    }
}

#[test]
fn test_empty_history_returns_none() {
    let classifier = ResidualDecayClassifier::default();
    let result = classifier.classify(&[]);
    assert!(result.is_none());
}
```

---

## 🔒 The Constitution

ArqonHPO operates under a [Constitution](constitution.md) that defines inviolable principles:

> **"Code that violates the Constitution will not be merged."**

Key principles:

1. **No Happy Path Testing** — Every edge case must have a test
2. **No Silent Failures** — All errors must surface with context
3. **Zero Unbounded Growth** — Memory and CPU must be bounded
4. **Audit Everything** — Every state change must be logged

If your PR touches safety-critical code, expect thorough review. This is a feature, not a bug.

---

## 💬 Communication

### Before Starting Major Work

Open an issue or discussion first! We want to:

- Ensure the feature aligns with our roadmap
- Provide early design feedback
- Prevent duplicate work

### Getting Help

| Channel                                                                     | Use For                              |
| --------------------------------------------------------------------------- | ------------------------------------ |
| [GitHub Discussions](https://github.com/novelbytelabs/ArqonHPO/discussions) | Questions, ideas, design discussions |
| [Issue Comments](https://github.com/novelbytelabs/ArqonHPO/issues)          | Specific issue discussion            |
| PR Comments                                                                 | Code review, implementation details  |

### Response Times

- **Issues:** Usually within 48 hours
- **PRs:** Initial review within 72 hours
- **Discussions:** We try to respond same-day

---

## 🎖️ Recognition

### All Contributors

Every contributor is recognized:

- Listed in `CONTRIBUTORS.md`
- Mentioned in release notes
- GitHub contributor badge

### Core Contributors

Sustained contributors may be invited to:

- Join the maintainer team
- Get write access to the repository
- Participate in roadmap planning

---

## 📜 License

ArqonHPO is licensed under the Apache License, Version 2.0.

By contributing, you agree that your contributions will be licensed under the same terms.

```
                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

Copyright 2024-2026 Novel Byte Labs

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

### Developer Certificate of Origin

We use the [Developer Certificate of Origin (DCO)](https://developercertificate.org/). By submitting a PR, you certify that you wrote the code or have the right to submit it under the Apache 2.0 license.

---

## 🙏 Thank You

Open source is built by people like you, in moments stolen from busy lives, driven by the belief that we can make something better together.

Every contribution matters. Every test you write. Every typo you fix. Every question you ask that helps us improve our docs.

**You're not just writing code. You're building the foundation for the next generation of intelligent systems.**

Welcome to ArqonHPO. We're glad you're here.

---

<div align="center">

**Ready to contribute?**

[Browse Issues](https://github.com/novelbytelabs/ArqonHPO/issues) • [Read the Docs](https://novelbytelabs.github.io/ArqonHPO) • [Join Discussions](https://github.com/novelbytelabs/ArqonHPO/discussions)

</div>
