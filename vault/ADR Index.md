---
title: "ADR Index"
type: hub
layer: architecture
tags:
  - type/hub
  - type/adr
summary: "Index of Architecture Decision Records (ADRs) for FerroTex."
---

# Architecture Decision Records (ADRs)

Key architectural decisions governing FerroTex:

- [[docs/adrs/0001-typed-event-ir.md|ADR 0001: Typed Event IR]] — Structuring compiler and engine diagnostic output into typed stream events.
- [[docs/adrs/0002-uncertainty-as-first-class.md|ADR 0002: Uncertainty as First-Class]] — Modeling incomplete or speculative parsing states explicitly.
- [[docs/adrs/0003-thin-extension-thick-server.md|ADR 0003: Thin Extension Thick Server]] — Keeping editor extensions minimal while consolidating logic in `ferrotexd`.
- [[docs/adrs/0004-streaming-incremental-parsing.md|ADR 0004: Streaming Incremental Parsing]] — High performance incremental LaTeX parsing.
- [[docs/adrs/0005-universal-build-dag.md|ADR 0005: Universal Build DAG]] — Directed acyclic graph execution for multi-pass compilation.
- [[docs/adrs/0006-tectonic-stepping-io.md|ADR 0006: Tectonic Stepping I/O]] — Stepping I/O hooks for Tectonic engine integration.
- [[docs/adrs/0007-multi-platform-vsix.md|ADR 0007: Multi-Platform VSIX]] — Cross-platform bundling for VS Code.

Connected to [[FerroTex Overview|FerroTex Overview]] and [[Architecture Overview|Architecture Overview]].
