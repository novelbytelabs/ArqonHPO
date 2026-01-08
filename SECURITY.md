# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Do NOT open a public issue for security vulnerabilities.**

Email: security@arqon.dev

We aim to respond within 48 hours and will work with you to understand and address the issue.

## Security Practices

- **SLSA Level 3:** Build provenance for all releases.
- **SBOM:** CycloneDX Software Bill of Materials for each release.
- **Fuzzing:** Continuous fuzzing of config parsing via `cargo-fuzz`.
- **Dependency Auditing:** `cargo audit` in CI.

## Audit Exceptions

Temporary `cargo audit` ignores are documented in `.cargo/audit.toml` and must include an expiry date and rationale.
Current exception: `RUSTSEC-2023-0071` (rsa timing advisory) is ignored until 2025-06-30 while upstream fixes are pending.
