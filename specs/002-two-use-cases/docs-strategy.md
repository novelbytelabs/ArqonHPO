# Strategy: Documentation "The Gamut"

**Goal**: Zero friction for simulation engineers and ML engineers.

## 1. The Five Pillars
We adopt an expanded Diátaxis + Community framework:
1.  **Tutorials** (Learning): Quickstart & "First Run".
2.  **How-to** (Problem-solving): The Cookbook & Runbooks.
3.  **Reference** (Information): API Docs, Manpages, Technotes.
4.  **Explanation** (Understanding): Theory & Architecture (ADRs).
5.  **Community** (Collaboration): Contributing, Governance, Standards.

## 2. Technical Stack
- **Site Generator**: `mkdocs-material` (Python/General).
- **Embedded Rust Docs**: `mdbook` (optional integration).
- **API Auto-gen**:
  - Python: `mkdocstrings` + `griffe`.
  - Rust: `cargo doc` (linked from main site).
- **Versioning**: `mike` to serve `/v1/`, `/v2/`, and `/latest/`.

## 3. Deliverables Matrix

### A. Root & Community (GitHub Native)
- **`README.md`**: The "Box Cover". Value prop, badges, 1-minute install.
- **`CONTRIBUTING.md`**: Dev setup, testing commands, PR process.
- **`CODE_OF_CONDUCT.md`**: Standard covenant.
- **`SECURITY.md`**: Reporting vulnerabilities.
- **`.github/ISSUE_TEMPLATE/`**: Bug reports, Feature requests.
- **`.github/PULL_REQUEST_TEMPLATE.md`**: Checklist for contributors.

### B. The Operations Manual (Runbooks & Technotes)
- **Runbooks**: "How to Release", "How to Debug CI", "Emergency Patching".
- **Technotes**: Deep technical dives (e.g., "TN-001: RNG Stability Analysis", "TN-002: FFI Safety Boundaries").

### C. The Cookbook (High-Value Content)
The differentiator. It will contain runnable recipes for:
- "Tuning a PyTorch Lightning Module"
- "Tuning a Black-box Bash Script"
- "Handling Failures & Retries"
- "Visualizing the Probe Phase"

