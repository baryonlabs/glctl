# Security Policy

## Reporting a vulnerability

Please report security issues **privately**, not as public GitHub issues.

- **Email**: security@baryonlabs.io (or the address listed on the maintainer's GitHub profile)
- **GitHub Security Advisory**: https://github.com/baryonlabs/glctl/security/advisories/new

We aim to acknowledge reports within **3 business days** and to ship a fix or coordinated disclosure within **30 days** for valid reports.

## Scope

Issues in scope:

- Path traversal or arbitrary file write through any `glctl` subcommand
- Lineage tampering — rewriting or forging generations another user owns
- Network handshake issues with glhub (`glctl push`)
- Memory safety: any panic that could be turned into a crash-DoS in batch mode
- Dependency vulnerabilities surfaced by `cargo audit`

Issues that are **NOT** typically in scope:

- Hardening suggestions for shell-script wrappers around glctl (those belong in your wrapper)
- Issues that require a malicious operator on the same machine
- Disagreements with output formatting

## Supported versions

Latest tagged release on `main`. Older versions: upgrade is the supported fix path.

## Coordinated disclosure

We'll work on a disclosure timeline with you and credit reporters in release notes unless they prefer to stay anonymous.
