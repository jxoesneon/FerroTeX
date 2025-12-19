# Release Process

This document defines the intended release process for FerroTeX.

## Versioning

- Semantic Versioning is intended once:
  - the log event IR schema is stable
  - the LSP behaviors are stable

## Release Artifacts (planned)

- Rust crates:
  - `ferrotexd` (server)
  - `ferrotex-cli` (offline parser)
- VS Code extension package
- Documentation snapshot

## Pre-Release Checklist

- Update `CHANGELOG.md`
- Ensure docs are consistent with implemented behavior
- Run:
  - unit tests
  - golden tests
  - benchmark suite
- Verify schema versioning rules

## Publishing

- Tag release in Git
- Publish GitHub release notes
- Publish extension (once configured)

## Backwards Compatibility Policy (target)

- Breaking schema changes are allowed before `1.0` but must be documented.
- After `1.0`, breaking changes require:
  - version bump
  - migration notes
