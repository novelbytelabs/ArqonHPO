# CI/CD Runbook

This guide details the Continuous Integration and Deployment (CI/CD) pipeline for ArqonHPO, managed via GitHub Actions.

## 👋 Quick Reference

| Action | Command / Trigger |
| :--- | :--- |
| **Run Tests Locally** | `cargo test --workspace` |
| **Check MSRV** | `manual check required` (see below) |
| **Fix Formatting** | `cargo fmt --all` |
| **Run Clippy** | `cargo clippy --workspace` |
| **Trigger Dependabot** | Comment `@dependabot rebase` on PR |
| **Cut a Release** | `git tag v0.2.x && git push origin v0.2.x` |

---

## 🏗 Pipeline Architecture

Our pipeline (`.github/workflows/ci.yml`) enforces quality gates on every Pull Request and merge to `main`.

### 1. Cross-Platform Rust Matrix
We compile and test on native runners to ensure OS compatibility:
- **Linux** (`ubuntu-latest`)
- **macOS** (`macos-latest`)
- **Windows** (`windows-latest`)

### 2. Minimum Supported Rust Version (MSRV)
* **Current MSRV:** `1.82.0`
* **Check:** Builds `cargo build` using strict version 1.82 to prevent accidental usage of newer features.
* **Troubleshooting:** If this fails but stable passes, you likely used a new Rust feature or updated a dependency to a version that requires a newer Rust.
    * *Fix:* Downgrade the dependency or pin it (e.g., `criterion = "=0.5.1"`).

### 3. Python Compatibility Matrix
We test bindings against supported Python versions:
- `3.10`
- `3.11`
- `3.12`

### 4. Code Quality & Benchmarks
- **Coverage:** Runs `cargo tarpaulin` and uploads to Codecov. (Excludes `arqonhpo` bindings due to FFI limits).
- **Benchmarks:** Runs `cargo bench` to detect performance regressions.
- **Docs:** Builds the MkDocs site to ensure no broken links or missing plugins.

---

## 🤖 Dependabot Runbook

Dependabot automatically opens PRs to update dependencies.

### 🔄 How to Rebase
If a Dependabot PR is showing red checks because `main` has changed:
1. Open the PR.
2. Comment:
   ```text
   @dependabot rebase
   ```
3. Wait for it to rebuild.

### ❌ Dealing with MSRV Conflicts
Sometimes Dependabot bumps a crate to a version that drops support for our MSRV (e.g., bumping `criterion` to `0.8.1` which needs Rust 1.86).

**Solution:**
1. **Identify:** The CI failure will explicitly say `package requires rustc X.XX`.
2. **Pin:** Explicitly pin the older version in `Cargo.toml`.
   ```toml
   [dev-dependencies]
   criterion = "=0.5.1"  # Pinned for MSRV 1.82 compatibility
   ```
3. **Close:** Manually close the Dependabot PR.

---

## 🚀 Release Process

Releases are automated via `.github/workflows/release.yml`.

### Steps:
1. **Update Version:** Bump version in `Cargo.toml` and run `cargo build` to update `Cargo.lock`.
2. **Commit:** Merge version bump to `main`.
3. **Tag:**
   ```bash
   git tag v0.3.0
   git push origin v0.3.0
   ```
4. **Automation:** GitHub Actions will:
   - Build binaries for Linux, macOS, and Windows.
   - Build Python wheels (manylinux, macos, windows).
   - Generate SLSA Provenance attestation.
   - Generate SBOM (Software Bill of Materials).
   - Create a GitHub Release with artifacts attached.

### Security
* **SLSA Level 3:** We generate provenance for all build artifacts to prevent supply chain attacks.
* **SBOM:** A CycloneDX Software Bill of Materials is generated for every release.

---

## 🛠 Local Development Commands

Before pushing, it's good practice to run:

```bash
# 1. Format code
cargo fmt --all

# 2. Catch common mistakes
cargo clippy --workspace --all-targets -- -D warnings

# 3. Run tests
cargo test --workspace

# 4. Run documentation site locally
mkdocs serve
```
