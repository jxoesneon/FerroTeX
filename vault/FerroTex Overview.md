---
title: "FerroTex Overview"
type: hub
layer: root
tags:
  - type/hub
  - layer/root
  - project/ferrotex
summary: "Central hub for the FerroTex project knowledge graph. A high-performance LaTeX IDE engine written in Rust."
---

# FerroTex Architectural Graph

FerroTex is a modern, modular, high-performance LaTeX and BibTeX authoring ecosystem, featuring a modular Language Server Protocol (`ferrotexd`), integrated Debug Adapter Protocol (`ferrotex-dap`), comprehensive build system, math semantic validation, and modern editor integrations.

## Cognitive Brain & Memory
- [[Brain/Active Context|Active Cognitive Context]] — Current working state and session invariants.
- [[Brain/Memory Stream|Episodic Memory Stream]] — Chronological log of operations and decisions.
- [[Brain/Task Register|Task Register]] — Objectives, active mandates, and backlog.

## Core Hubs & Navigation
- [[Crates Index|All 10 Crates Index]]
- [[Architecture Overview|System Architecture & Pipelines]]
- [[Specs Index|Technical Specifications]]
- [[ADR Index|Architectural Decision Records]]
- [[Cargo.toml|Root Cargo Workspace Manifest]]
- [[README.md|Repository Readme]]

## Workspace Subsystems
1. **Language Server & Daemon**: [[crates/ferrotexd/src/lib.rs|ferrotexd]], [[crates/ferrotexd/src/main.rs|ferrotexd binary]]
2. **Grammar & Parsing**: [[crates/ferrotex-syntax/src/lib.rs|ferrotex-syntax]], [[crates/ferrotex-syntax/src/parser.rs|syntax parser]]
3. **Core Validation & Package Management**: [[crates/ferrotex-core/src/lib.rs|ferrotex-core]]
4. **Build & Toolchain Orchestration**: [[crates/ferrotex-build/src/lib.rs|ferrotex-build]]
5. **Log Event IR & Parsing**: [[crates/ferrotex-log/src/lib.rs|ferrotex-log]]
6. **Math Semantics Engine**: [[crates/ferrotex-math-semantics/src/lib.rs|ferrotex-math-semantics]]
7. **Package Scanner & Indexer**: [[crates/ferrotex-package/src/lib.rs|ferrotex-package]]
8. **Static Code Analysis**: [[crates/ferrotex-analysis/src/lib.rs|ferrotex-analysis]]
9. **Debug Adapter Protocol**: [[crates/ferrotex-dap/src/lib.rs|ferrotex-dap]]
10. **Command Line Interface**: [[crates/ferrotex-cli/src/lib.rs|ferrotex-cli]]
11. **Editor Extensions**: [[editors/vscode/src/extension.ts|VS Code Extension]]
