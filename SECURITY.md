# Security Policy

## Supported Versions

FerroTeX is currently in a pre-implementation / early development stage.

Security fixes will be published as part of normal releases once release automation exists.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately.

- Preferred: open a GitHub Security Advisory (once the repository is hosted)
- Alternative: email the maintainer (to be added)

Please include:

- a clear description of the issue
- reproduction steps
- impacted versions/commits (if known)
- potential impact

## Threat Model Notes

FerroTeX interacts with:

- untrusted `.tex` inputs
- external TeX engines and toolchains
- potentially hostile log content (crafted output)

Security-sensitive areas include:

- command execution (engine runner)
- path handling and workspace traversal
- parsing robustness (panic-free parsing, memory bounds)
- VS Code extension IPC boundaries

## Responsible Disclosure

We will acknowledge reports, provide a timeline when possible, and credit reporters if desired.
