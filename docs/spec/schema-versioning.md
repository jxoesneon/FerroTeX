# Schema Versioning

## Status

- **Type:** Normative
- **Stability:** Draft

## Scope

This document specifies versioning rules for:

- the Event IR (`docs/spec/log-event-ir.md`)
- the diagnostic record emitted by the server/CLI

## Goals

- Allow the Rust core to evolve while keeping consumers safe.
- Prevent silent breakage of downstream tools.

## Version Field

Every IR output MUST include:

- `schema_version` (string)

## Compatibility Rules

### Pre-1.0 (`0.x`)

- Breaking changes are permitted.
- Breaking changes MUST be documented in `CHANGELOG.md`.

### 1.0 and beyond

- Backwards compatible additions:
  - new event kinds
  - new optional fields
  - new diagnostic codes

- Breaking changes:
  - removing or renaming fields
  - changing semantics of existing fields
  - changing default meaning of ranges/paths

Breaking changes MUST bump major version.

## Consumer Requirements

Consumers MUST:

- tolerate unknown event kinds
- ignore unknown fields
- treat absence of optional fields as expected
