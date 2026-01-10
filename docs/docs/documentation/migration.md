# Migration Guide

This guide helps you upgrade between ArqonHPO versions.

---

## v0.2 → v0.3 (Current)

### Breaking Changes

1. **Config field rename:** `probe_budget` → `probe_ratio`

```diff
{
  "seed": 42,
  "budget": 100,
- "probe_budget": 20,
+ "probe_ratio": 0.2,
}
```

2. **Python import path change:**

```diff
- from arqonhpo._internal import ArqonSolver
+ from arqonhpo import ArqonSolver
```

3. **Artifact schema:** `history` field now includes `cost`:

```diff
{
  "eval_id": 0,
  "params": {"x": 1.0},
  "value": 0.5,
+ "cost": 1.0
}
```

### Migration Steps

1. Update config files (rename `probe_budget` to `probe_ratio`)
2. Update imports in Python code
3. Re-export artifacts from v0.2 state files:
   ```bash
   # With v0.2
   arqonhpo export --state old_state.json --output artifact.json
   
   # Manually add "cost": 1.0 to each history entry
   
   # With v0.3
   arqonhpo import --artifact artifact.json --state new_state.json
   ```

---

## v0.1 → v0.2

### Breaking Changes

1. **CLI renamed:** `arqon` → `arqonhpo`

```diff
- arqon run --config config.json
+ arqonhpo run --config config.json
```

2. **Python package renamed:**

```diff
- pip install arqon
+ pip install arqonhpo
```

3. **Config schema overhaul:** Complete rewrite
   - v0.1 configs are **not compatible**
   - Re-create configs using v0.2 schema

---

## Version Compatibility Matrix

| ArqonHPO | Python | Rust | State Format |
|----------|--------|------|--------------|
| v0.3.x | 3.10+ | 1.82+ | v3 |
| v0.2.x | 3.9+ | 1.75+ | v2 |
| v0.1.x | 3.8+ | 1.70+ | v1 (incompatible) |

---

## State File Migration

State files are **not forward compatible**. To migrate state:

1. Export from old version:
   ```bash
   # Using OLD arqonhpo version
   arqonhpo export --state state.json --output artifact.json
   ```

2. Review artifact for schema changes

3. Import with new version:
   ```bash
   # Using NEW arqonhpo version
   arqonhpo import --artifact artifact.json --state new_state.json
   ```

> [!WARNING]
> If import fails due to schema changes, you may need to manually edit the artifact JSON.

---

## Deprecation Notices

### v0.3 (Current)

| Feature | Status | Replacement |
|---------|--------|-------------|
| `probe_budget` config | Removed | Use `probe_ratio` |
| `arqonhpo._internal` imports | Removed | Use `arqonhpo` |

### v0.4 (Upcoming)

| Feature | Status | Replacement |
|---------|--------|-------------|
| `strategy_params.alpha` | Deprecated | Use `strategy_params.reflection_coeff` |
| `--log-level debug` | Deprecated | Use `--log-level=debug` (with `=`) |

---

## Getting Help

If you encounter migration issues:

1. Check [Troubleshooting](troubleshooting.md)
2. Open a [GitHub Issue](https://github.com/novelbytelabs/ArqonHPO/issues)
3. Ask in [Discussions](https://github.com/novelbytelabs/ArqonHPO/discussions)

---

## Next Steps

- [Installation](installation.md) — Install latest version
- [Quickstart](quickstart.md) — Get running with v0.3
- [Changelog](../project/changelog.md) — Full version history
