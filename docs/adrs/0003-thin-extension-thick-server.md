# ADR 0003: Thin VS Code Extension, Thick Rust Server

- **Status:** Accepted
- **Date:** 2025-12-18

## Context

Splitting logic between extension and server risks duplication:

- two parsing implementations
- divergent heuristics
- inconsistent diagnostics

Rust provides safety and performance; TypeScript provides editor UX.

## Decision

Keep the VS Code extension thin:

- lifecycle management
- configuration plumbing
- UI rendering

All parsing, reconstruction, and diagnostic mapping logic lives in Rust.

## Alternatives Considered

- Implement parsing in TypeScript
  - rejected: performance and robustness concerns; duplication with Rust core

- Hybrid parsing (some heuristics in extension)
  - rejected: increases inconsistency risk

## Consequences

- Better testability and determinism.
- Extension becomes mostly a transport and UX layer.
