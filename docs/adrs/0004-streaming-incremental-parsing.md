# ADR 0004: Streaming and Incremental Parsing as a Core Requirement

- **Status:** Accepted
- **Date:** 2025-12-18

## Context

TeX logs can be large, and editor feedback is most valuable when incremental.

Reparsing an entire log after every change is wasteful and can cause diagnostic flapping.

## Decision

Design the parser and reconstruction to support:

- streaming ingestion (tailing `.log` and/or capturing stdout)
- incremental updates using synchronization anchors

The system must preserve stability:

- partial structures may exist; do not emit unstable remappings without confidence gating.

## Alternatives Considered

- Always parse offline after compilation completes
  - rejected: poor latency, no live feedback

- Always reparse from byte 0
  - rejected: unnecessary work, poor scalability

## Consequences

- Parser must maintain state checkpoints.
- Evaluation must include incremental latency metrics.
