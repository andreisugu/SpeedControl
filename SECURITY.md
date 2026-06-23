# Security Policy

We take the security of `SpeedControl` seriously. This document outlines how to report vulnerabilities and describes our dependency auditing practices.

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please do **not** open a public issue. Instead, report it using one of the following methods:

1. **Private Vulnerability Reporting:** If this repository is hosted on GitHub, you can use the "Report a vulnerability" button under the **Security** tab to open a private draft advisory.
2. **Email:** You can contact the maintainers privately at `security@example.com` (replace with actual maintainer contact).

We will investigate all reports promptly and coordinate a fix before disclosing the issue publicly.

## Auditing and Supply Chain Security

To protect our users and maintain integrity, the project is configured with automated security gates:
- **`cargo-deny`**: Automatically blocks dependency updates that introduce unapproved registries, non-permissive licenses, or duplicate crate version bloat.
- **`cargo-audit`**: Scans the dependency tree against the RustSec Advisory Database on every pull request to catch crates with known security vulnerabilities.
- **CI/CD Actions**: GitHub Actions workflows run format check, clippy warning check, cargo deny scan, cargo audit scan, and native unit tests on every push.
