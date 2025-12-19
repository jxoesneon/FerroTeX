# ADR 0001: Typed Event IR for TeX Logs

- **Status:** Accepted
- **Date:** 2025-12-18

## Context

TeX engine logs are unstructured. Regex approaches tend to encode parsing knowledge implicitly and are difficult to evolve.

FerroTeX requires:

- provenance-preserving parsing (byte spans)
- extensibility for new constructs
- compatibility across engines/distributions

## Decision

Define a **typed event intermediate representation (IR)** as the canonical internal and external representation of parsed logs.

Key requirements:

- every event has a log byte span
- event kinds form a discriminated union
- schema version is explicit

See `docs/spec/log-event-ir.md`.

## Alternatives Considered

- Line-based structured records only
  - rejected: log semantics are not line-oriented

- Unversioned ad-hoc JSON
  - rejected: breaks consumers silently

## Consequences

- Parser and LSP layers can evolve independently.
- Consumers must tolerate unknown event kinds.
- Schema versioning becomes part of release discipline.
