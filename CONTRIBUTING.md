# Contributing to ArqonHPO

Thank you for your interest in contributing! 🎉

## Quick Start

```bash
git clone https://github.com/arqon/ArqonHPO.git
cd ArqonHPO
just build
just test
```

## Development Workflow

1. **Fork & Clone** the repository.
2. **Create a branch:** `git checkout -b feature/my-feature`
3. **Make changes** and add tests.
4. **Run checks:** `just check && just test`
5. **Submit a PR.**

## Code Style

- **Rust:** `rustfmt` + `clippy` (warnings = errors).
- **Python:** `ruff` + `mypy`.

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(core): add support for custom strategies
fix(bindings): handle empty history edge case
docs(cookbook): add PyTorch recipe
```

## Questions?

Open a [Discussion](https://github.com/arqon/ArqonHPO/discussions) or ask in the PR.
